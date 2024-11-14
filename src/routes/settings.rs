use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SettingsUpdateError {
    PasswordsNotMatch,
    Successful,
    ValidationError(String),
}

#[tracing::instrument]
#[server(SettingsUpdateAction, "/api")]
pub async fn settings_update(
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<SettingsUpdateError, ServerFnError> {
    let user = get_user().await?;
    let username = user.username();
    
    let user = match update_user_validation(user, image, bio, email, password, &confirm_password) {
        Ok(x) => x,
        Err(x) => return Ok(x),
    };
    
    user.update()
        .await
        .map(|_| SettingsUpdateError::Successful)
        .map_err(move |x| {
            tracing::error!("Problem updating user {}: {}", username, x);
            ServerFnError::ServerError("Problem updating user".into())
        })
}

#[cfg(feature = "ssr")]
fn update_user_validation(
    mut user: crate::models::User,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: &str,
) -> Result<crate::models::User, SettingsUpdateError> {
    // Handle password update if provided
    if !password.is_empty() {
        if password != confirm_password {
            return Err(SettingsUpdateError::PasswordsNotMatch);
        }
        let updated_user = user.clone().set_password(password)
            .map_err(SettingsUpdateError::ValidationError)?;
        user = updated_user;
    }

    // Update other fields
    user.set_email(email)
        .map_err(SettingsUpdateError::ValidationError)?;
    user.set_bio(bio)
        .map_err(SettingsUpdateError::ValidationError)?;
    user.set_image(image)
        .map_err(SettingsUpdateError::ValidationError)?;

    Ok(user)
}

#[cfg(feature = "ssr")]
async fn get_user() -> Result<crate::models::User, ServerFnError> {
    let Some(user_id) = crate::auth::get_user_id() else {
        leptos_axum::redirect("/login");
        return Err(ServerFnError::ServerError("Authentication required".into()));
    };

    crate::models::User::get(user_id).await.map_err(|e| {
        tracing::error!("Error fetching user: {}", e);
        ServerFnError::ServerError(e.to_string())
    })
}

#[tracing::instrument]
#[server(SettingsGetAction, "/api", "GetJson")]
pub async fn settings_get() -> Result<crate::models::User, ServerFnError> {
    get_user().await
}

#[component]
pub fn Settings(logout: crate::auth::LogoutSignal) -> impl IntoView {
    let settings_resource = create_resource(|| (), |_| async move { settings_get().await });

    view! {
        <Title text="Settings"/>

        <div class="settings-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Your Settings"</h1>

                        <Suspense fallback=move || view! { <div class="loading">"Loading settings..."</div> }>
                            <ErrorBoundary
                                fallback=|errors| view! {
                                    <div class="error-messages">
                                        <p>"Error loading settings: "</p>
                                        <pre>{format!("{errors:?}")}</pre>
                                    </div>
                                }
                            >
                                {move || {
                                    settings_resource.get().map(|result| {
                                        match result {
                                            Ok(user) => view! { <SettingsViewForm user/> }.into_view(),
                                            Err(e) => view! { 
                                                <div class="error-messages">
                                                    "Failed to load settings: " {e.to_string()}
                                                </div> 
                                            }.into_view()
                                        }
                                    })
                                }}
                            </ErrorBoundary>
                        </Suspense>

                        <hr/>
                        <ActionForm action=logout>
                            <button type="submit" class="btn btn-outline-danger">
                                "Click here to logout"
                            </button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SettingsViewForm(user: crate::models::User) -> impl IntoView {
    let settings_action = create_server_action::<SettingsUpdateAction>();
    let pending = settings_action.pending();
    let result = settings_action.value();

    let has_error = move || {
        result.with(|r| {
            matches!(r, Some(Ok(SettingsUpdateError::PasswordsNotMatch | SettingsUpdateError::ValidationError(_))) | Some(Err(_)))
        })
    };

    let success = move || {
        result.with(|r| matches!(r, Some(Ok(SettingsUpdateError::Successful))))
    };

    let is_pending = move || pending.get();

    view! {
        <div class="settings-form" class:has-error=has_error>

        {move || {
            if success() {
                view! {
                    <div class="alert alert-success" role="alert">
                        "Your settings have been successfully updated!"
                    </div>
                }.into_view()
            } else {
                view! { <div></div> }.into_view()
            }
        }}

            {move || result.get().map(|r| {
                match r {
                    Ok(SettingsUpdateError::Successful) => view! {
                        <div class="alert alert-success">"Settings updated successfully!"</div>
                    },
                    Ok(SettingsUpdateError::ValidationError(msg)) => view! {
                        <div class="alert alert-danger">{msg}</div>
                    },
                    Ok(SettingsUpdateError::PasswordsNotMatch) => view! {
                        <div class="alert alert-danger">"Passwords do not match"</div>
                    },
                    Err(e) => view! {
                        <div class="alert alert-danger">{format!("Error: {}", e)}</div>
                    }
                }
            })}

            <ActionForm action=settings_action>
                <fieldset disabled=move || is_pending()>
                    <fieldset class="form-group">
                        <input
                            name="image"
                            value=user.image()
                            class="form-control"
                            type="text"
                            placeholder="URL of profile picture"
                        />
                    </fieldset>

                    <fieldset class="form-group">
                        <input
                            disabled
                            value=user.username()
                            class="form-control form-control-lg"
                            type="text"
                            placeholder="Username"
                        />
                    </fieldset>

                    <fieldset class="form-group">
                        <textarea
                            name="bio"
                            class="form-control form-control-lg"
                            rows="8"
                            placeholder="Short bio about you"
                            prop:value=user.bio().unwrap_or_default()
                        />
                    </fieldset>

                    <fieldset class="form-group">
                        <input
                            name="email"
                            value=user.email()
                            class="form-control form-control-lg"
                            type="email"
                            placeholder="Email"
                        />
                    </fieldset>

                    <fieldset class="form-group">
                        <input
                            name="password"
                            class="form-control form-control-lg"
                            type="password"
                            placeholder="New Password"
                        />
                        <input
                            name="confirm_password"
                            class="form-control form-control-lg"
                            type="password"
                            placeholder="Confirm New Password"
                        />
                    </fieldset>

                    <button
                        class="btn btn-lg pull-xs-right"
                        // We can use has_error to change button styling
                        class:btn-primary=move || !has_error()
                        class:btn-danger=has_error
                        type="submit"
                        disabled=move || is_pending()
                    >
                        {move || {
                            if is_pending() { 
                                "Updating..." 
                            } else if has_error() {
                                "Try Again"
                            } else {
                                "Update Settings"
                            }
                        }}
                    </button>
                </fieldset>
            </ActionForm>
        </div>
    }
}