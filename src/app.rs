use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use crate::auth::{use_auth, AuthProvider, LogoutSignal, LoginSignal, SignupSignal};
use crate::auth::{LogoutAction, LoginAction, SignupAction};
use crate::components::NavItems;
use crate::routes::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <AuthProvider>
            <AppContent/>
        </AuthProvider>
    }
}

#[component]
fn AppContent() -> impl IntoView {
    provide_meta_context();

    let auth = use_auth();
    let user_id = auth.user_id;

    let logout: LogoutSignal = create_server_action::<LogoutAction>();
    let login: LoginSignal = create_server_action::<LoginAction>();
    let signup: SignupSignal = create_server_action::<SignupAction>();

    // Track auth state changes from server actions
    let auth_state = create_memo(move |_| {
        // Depend on all action versions to re-run when any auth action completes
        logout.version().get();
        login.version().get();
        signup.version().get();
        crate::auth::get_user_id()
    });

    // Update UI when auth state changes
    create_effect(move |_| {
        if let Some(new_state) = auth_state.get() {
            user_id.set(Some(new_state));
        } else {
            user_id.set(None); // Ensure we clear the user ID when logged out
        }
    });

    view! {
        // External stylesheets
        <Stylesheet id="ionicons" href="https://code.ionicframework.com/ionicons/2.0.1/css/ionicons.min.css"/>
        <Stylesheet id="google-fonts" href="https://fonts.googleapis.com/css?family=Titillium+Web:700|Source+Serif+Pro:400,700|Merriweather+Sans:400,700|Source+Sans+Pro:400,300,600,700,300italic,400italic,600italic,700italic"/>
        <Stylesheet id="main-css" href="https://demo.productionready.io/main.css"/>
        <Stylesheet id="app-css" href="/pkg/thedeveloper-leptos.css"/>

        <Title text="Welcome to Leptos"/>

        <Router>
            // Navigation
            <nav class="navbar navbar-light">
                <div class="container">
                    <A class="navbar-brand" href="/" exact=true>
                        "thedeveloper"
                    </A>
                    <ul class="nav navbar-nav pull-xs-right">
                        <NavItems user_id=user_id/>
                    </ul>
                </div>
            </nav>

            // Main content
            <main>
                <Routes>
                    <Route 
                        path="/" 
                        view=move || view! { <HomePage user_id=user_id/> }
                    />
                    <Route 
                        path="/login" 
                        view=move || view! { <Login login/> }
                    />
                    <Route 
                        path="/signup" 
                        view=move || view! { <Signup signup/> }
                    />
                    <Route 
                        path="/settings" 
                        view=move || view! { <Settings logout/> }
                    />
                    <Route 
                        path="/editor/:slug?" 
                        view=|| view! { <Editor/> }
                    />
                    <Route 
                        path="/article/:slug" 
                        view=move || view! { <Article user_id=user_id/> }
                    />
                    <Route 
                        path="/profile/:user_id" 
                        view=move || view! { <Profile user_id=user_id/> }
                    />
                </Routes>
            </main>

            // Footer
            <footer>
                <div class="container">
                    <A href="/" class="logo-font">
                        "thedeveloper"
                    </A>
                    <span class="attribution">
                        "Empowering Developers, One Line of Code at a Time. "
                        <a href="https://thinkster.io">"Thinkster"</a>
                        ". Code & design licensed under MIT."
                    </span>
                </div>
            </footer>
        </Router>
    }
}