use leptos::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

#[tracing::instrument]
pub fn validate_signup(
    name: String,
    email: String,
    password: String,
) -> Result<crate::models::User, String> {
    if name.trim().is_empty() {
        return Err("Name cannot be empty".into());
    }

    let email = email.trim().to_lowercase();
    if !email.contains('@') || !email.contains('.') || email.len() < 5 {
        return Err("Invalid email format".into());
    }

    if password.len() < 8 {
        return Err("Password must be at least 8 characters".into());
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least one uppercase letter".into());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain at least one number".into());
    }

    crate::models::User::default()
        .set_name(name)?
        .set_password(password)?
        .set_email(email)
        .cloned()
}

#[tracing::instrument(skip(password))]
#[server(SignupAction, "/api")]
pub async fn signup_action(
    name: String,
    email: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::ServerError("Response context not available".into()))?;

    if name.is_empty() || email.is_empty() || password.is_empty() {
        response_options.set_status(axum::http::StatusCode::BAD_REQUEST);
        return Ok(SignupResponse::ValidationError(
            "All fields are required.".into(),
        ));
    }

    match validate_signup(name.clone(), email.clone(), password) {
        Ok(mut user) => match user.insert().await {
            Ok(user_id) => {
                let token = crate::auth::server::generate_token(user_id);
                crate::auth::auth::set_auth_cookie(&response_options, &token);
                leptos_axum::redirect("/");
                Ok(SignupResponse::Success)
            }
            Err(sqlx::Error::Database(db_err)) if db_err.constraint().is_some() => {
                response_options.set_status(axum::http::StatusCode::CONFLICT);
                match db_err.constraint() {
                    Some("users_email_key") => Ok(SignupResponse::CreateUserError(
                        "Email already registered".into(),
                    )),
                    Some("users_username_key") => Ok(SignupResponse::CreateUserError(
                        "Username already taken".into(),
                    )),
                    _ => {
                        tracing::error!("Unexpected constraint violation: {:?}", db_err);
                        Ok(SignupResponse::CreateUserError(
                            "An unexpected error occurred".into(),
                        ))
                    }
                }
            }
            Err(err) => {
                tracing::error!("User insertion error: {:?}", err);
                response_options.set_status(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                Ok(SignupResponse::CreateUserError(
                    "Internal server error".into(),
                ))
            }
        },
        Err(validation_error) => {
            response_options.set_status(axum::http::StatusCode::BAD_REQUEST);
            Ok(SignupResponse::ValidationError(validation_error))
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum LoginMessages {
    Successful,
    Unsuccessful(String),
}

#[server(LoginAction, "/api")]
#[tracing::instrument(skip(password))]
pub async fn login_action(email: String, password: String) -> Result<LoginMessages, ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::ServerError("Response context not available".into()))?;

    if email.is_empty() || password.is_empty() {
        response_options.set_status(axum::http::StatusCode::BAD_REQUEST);
        return Ok(LoginMessages::Unsuccessful("All fields are required".into()));
    }

    match crate::auth::auth::authenticate(email, password).await? {
        LoginMessages::Successful => {
            leptos_axum::redirect("/");
            Ok(LoginMessages::Successful)
        }
        err => {
            response_options.set_status(axum::http::StatusCode::UNAUTHORIZED);
            Ok(err)
        }
    }
}

#[server(LogoutAction, "/api")]
#[tracing::instrument]
pub async fn logout_action() -> Result<(), ServerFnError> {
    crate::auth::auth::logout().await?;
    leptos_axum::redirect("/login");
    Ok(())
}

#[server(CurrentUserAction, "/api")]
#[tracing::instrument]
pub async fn current_user() -> Result<crate::models::User, ServerFnError> {
    let user_id = crate::auth::get_user_id()
        .ok_or_else(|| ServerFnError::ServerError("Not authenticated".into()))?;

    crate::models::User::get_by_id(user_id)
        .await
        .map_err(|err| {
            tracing::error!("Failed to retrieve user: {:?}", err);
            ServerFnError::ServerError("Authentication error".into())
        })
}