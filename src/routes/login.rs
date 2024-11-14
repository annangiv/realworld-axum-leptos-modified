use crate::auth::{LoginMessages, LoginSignal};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn Login(login: LoginSignal) -> impl IntoView {
    let result_of_call = login.value();

    // Improved state management using a single signal for error handling
    let (error_message, set_error_message) = create_signal(None::<String>);

    let error_class = move |is_invalid| {
        if is_invalid {
            "form-control form-control-lg is-invalid"
        } else {
            "form-control form-control-lg"
        }
    };

    // Observe the result of the login action and update the error message
    create_effect(move |_| {
        if let Some(msg) = result_of_call.get() {
            match msg {
                Ok(LoginMessages::Unsuccessful) => {
                    set_error_message.set(Some("Incorrect username or password".into()));
                }
                _ => {
                    set_error_message.set(None);
                }
            }
        }
    });

    view! {
        <Title text="Login"/>
        <div class="auth-page">
            <div class="container page">
                <div class="row">
                    <div class="col-md-6 offset-md-3 col-xs-12">
                        <h1 class="text-xs-center">"Login"</h1>
                        <p class="text-xs-center">
                            <A href="/signup">"Don't have an account?"</A>
                        </p>
                        <p class="error-messages text-xs-center">
                            {move || error_message.with(|msg| msg.clone().unwrap_or_default())}
                        </p>

                        <ActionForm action=login>
                            <fieldset class="form-group">
                                <input
                                    name="email"
                                    class=move || error_class(error_message.with(Option::is_some))
                                    type="text"
                                    placeholder="Your Email"
                                    aria-label="email"
                                    required=true
                                />
                            </fieldset>
                            <fieldset class="form-group">
                                <input
                                    name="password"
                                    class=move || error_class(error_message.with(Option::is_some))
                                    type="password"
                                    placeholder="Password"
                                    aria-label="Password"
                                    required=true
                                />
                            </fieldset>
                            <button class="btn btn-lg btn-primary pull-xs-right">"Sign in"</button>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}
