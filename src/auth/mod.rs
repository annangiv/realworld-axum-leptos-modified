use leptos::{Action, RwSignal, ServerFnError};
use uuid::Uuid;

mod api;
mod auth;
#[cfg(feature = "ssr")]
mod server;
#[cfg(not(feature = "ssr"))]
mod client;

pub use api::*;
pub use auth::{AuthProvider, use_auth, LoginSignal, LogoutSignal, SignupSignal};
#[cfg(not(feature = "ssr"))]
pub use client::get_user_id;
#[cfg(feature = "ssr")]
pub use server::*;

pub type LogoutSignal = Action<LogoutAction, Result<(), ServerFnError>>;
pub type LoginSignal = Action<LoginAction, Result<LoginMessages, ServerFnError>>;
pub type SignupSignal = Action<SignupAction, Result<SignupResponse, ServerFnError>>;
pub type UserIdSignal = RwSignal<Option<Uuid>>;