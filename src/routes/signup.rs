use crate::auth::{validate_signup, SignupAction, SignupResponse, SignupSignal};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn Signup(signup: SignupSignal) -> impl IntoView {
    let result_of_call = signup.value();
    let pending = signup.pending();
    
    // Signal to manage error messages
    let (error_message, set_error_message) = create_signal(None::<String>);

    let has_error = move || error_message.get().is_some();
    
    let success = move || {
        result_of_call.with(|r| matches!(r, Some(Ok(SignupResponse::Success))))
    };

    // Effect to watch the result of the signup call and update the error message
    create_effect(move |_| {
        if let Some(msg) = result_of_call.get() {
            match msg {
                Ok(SignupResponse::ValidationError(x)) => {
                    set_error_message.set(Some(format!("{x}")));
                }
                Ok(SignupResponse::CreateUserError(x)) => {
                    set_error_message.set(Some(format!("            {x}")));
                }
                Ok(SignupResponse::Success) => {
                    set_error_message.set(None); // Clear error on success
                }
                Err(ServerFnError::Deserialization(_)) => {
                    set_error_message.set(Some("Failed to deserialize data.".into()));
                }
                Err(x) => {
                    tracing::error!("Problem during signup: {x:?}");
                    set_error_message.set(Some("There was a problem, try again later".into()));
                }
            }
        }
    });

    let error_class = move |is_invalid| {
        if is_invalid {
            "form-control form-control-lg is-invalid"
        } else {
            "form-control form-control-lg"
        }
    };

    view! {
        <Title text="Signup"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Sign up"</h1>
                        <p class="text-xs-center">
                            <A href="/login">"Have an account?"</A>
                        </p>

                        // Show success message
                        {move || {
                            if success() {
                                view! {
                                    <div class="alert alert-success text-xs-center" role="alert">
                                        "Sign up successful! Redirecting..."
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        }}

                        // Show error messages
                        {move || {
                            if has_error() {
                                view! {
                                    <div class="alert alert-danger text-xs-center" role="alert">
                                        {error_message.get().unwrap_or_default()}
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        }}

                        <div class="signup-form" class:has-error=move || has_error()>
                            <ActionForm 
                                action=signup 
                                on:submit=move |ev| {
                                    let Ok(data) = SignupAction::from_event(&ev) else {
                                        return ev.prevent_default();
                                    };
                                    if let Err(x) = validate_signup(data.name.clone(), data.email.clone(), data.password.clone()) {
                                        set_error_message.set(Some(format!("Problem while validating: {x}")));
                                        result_of_call.set(Some(Ok(SignupResponse::ValidationError(x))));
                                        ev.prevent_default();
                                    }
                                }
                            >
                            <fieldset disabled=move || pending.get()>
                                <fieldset class="form-group">
                                    <input 
                                        name="name" 
                                        class=move || error_class(has_error()) 
                                        type="text" 
                                        placeholder="Your Full Name" 
                                        required=true
                                    />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input 
                                        name="email" 
                                        class=move || error_class(has_error()) 
                                        type="email" 
                                        placeholder="Email" 
                                        required=true
                                    />
                                </fieldset>
                                <fieldset class="form-group">
                                    <input 
                                        name="password" 
                                        class=move || error_class(has_error()) 
                                        type="password" 
                                        placeholder="Password" 
                                        required=true
                                    />
                                </fieldset>
                                <button 
                                    class="btn btn-lg pull-xs-right"
                                    class:btn-primary=move || !has_error()
                                    class:btn-danger=has_error
                                    type="submit"
                                    disabled=move || pending.get()
                                >
                                    {move || {
                                        if pending.get() {
                                            "Signing up..."
                                        } else if has_error() {
                                            "Try Again"
                                        } else {
                                            "Sign up"
                                        }
                                    }}
                                </button>
                            </fieldset>
                        </ActionForm>
                        </div>

                        // Additional guidance for errors
                        {move || {
                            if has_error() {
                                view! {
                                    <div class="form-help text-danger mt-3 text-xs-center">
                                        "Please correct the errors above and try again."
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}