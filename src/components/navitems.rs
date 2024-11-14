use crate::auth::*;
use leptos::*;
use leptos_router::*;
use leptos::logging::log;

#[component]
pub(crate) fn NavItems(user_id: UserIdSignal) -> impl IntoView {
    // Create a memo for efficient auth state tracking
    let auth_state = create_memo(move |_| {
        let id = user_id.get();
        log!("NavItems auth_state updated: {:?}", id);
        id
    });

    // Add effect to track auth state changes
    create_effect(move |_| {
        log!("Auth state changed - user_id: {:?}", user_id.get());
        log!("Auth state changed - auth_state: {:?}", auth_state.get());
    });
    
    let profile_label = move || {
        auth_state.get().map_or("Guest", |_| "My Account")
    };

    let profile_href = move || {
        auth_state.get().map_or(
            "/login".to_string(),
            |id| format!("/profile/{}", id)
        )
    };

    let is_logged_in = move || auth_state.get().is_some();

    view! {
        <li class="nav-item">
            <A class="nav-link" href="/" exact=true>
                <i class="ion-home"></i>
                " Home"
            </A>
        </li>

        <Show
            when=is_logged_in
            fallback=move || {
                view! {
                    <>
                        <li class="nav-item">
                            <A class="nav-link" href="/signup">
                                <i class="ion-plus-round"></i>
                                " Sign up"
                            </A>
                        </li>
                        <li class="nav-item">
                            <A class="nav-link" href="/login">
                                <i class="ion-log-in"></i>
                                " Login"
                            </A>
                        </li>
                    </>
                }
            }
        >
            <>
                <li class="nav-item">
                    <A class="nav-link" href="/editor">
                        <i class="ion-compose"></i>
                        " New Article"
                    </A>
                </li>
                <li class="nav-item">
                    <A class="nav-link" href="/settings">
                        <i class="ion-gear-a"></i>
                        " Settings"
                    </A>
                </li>
                <li class="nav-item">
                    <A class="nav-link" href=profile_href>
                        <i class="ion-person"></i>
                        {profile_label}
                    </A>
                </li>
            </>
        </Show>
    }
}