use crate::components::ArticlePreviewList;
use crate::components::ButtonFollow;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[server(UserArticlesAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn profile_articles(
    user_id: uuid::Uuid,
    favourites: Option<bool>,
) -> Result<Vec<crate::models::Article>, ServerFnError> {
    crate::models::Article::for_user_profile(user_id, favourites.unwrap_or_default())
        .await
        .map_err(|x| {
            let err = format!("Error while getting user_profile articles: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not retrieve articles, try again later".into())
        })
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserProfileModel {
    user: crate::models::User,
    following: Option<bool>,
}

#[server(UserProfileAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn user_profile(user_id: uuid::Uuid) -> Result<UserProfileModel, ServerFnError> {
    let user = crate::models::User::get_by_id(user_id).await.map_err(|x| {
        let err = format!("Error while getting user in user_profile: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not retrieve articles, try again later".into())
    })?;
    match crate::auth::get_user_id() {
        Some(logged_user_id) => sqlx::query!(
            "SELECT EXISTS(SELECT * FROM Follows WHERE follower_id=$1 and influencer_id=$2)",
            logged_user_id,
            user_id,
        )
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|x| {
            let err = format!("Error while getting user in user_profile: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not retrieve articles, try again later".into())
        })
        .map(|x| UserProfileModel {
            user,
            following: x.exists,
        }),
        None => Ok(UserProfileModel {
            user,
            following: None,
        }),
    }
}

#[component]
fn ProfileTabs(
    route_user_id: Memo<Option<uuid::Uuid>>,
    favourite: Memo<Option<bool>>,
) -> impl IntoView {
    let user_article_href = move || {
        format!(
            "/profile/{}",
            route_user_id.get().map_or("Unknown User".to_string(), |id| id.to_string())
        )
    };
    let favourites_href = move || format!("{}?favourites=true", user_article_href());

    view! {
        <div class="articles-toggle">
            <ul class="nav nav-pills outline-active">
                <li class="nav-item">
                    <a class="nav-link"
                        class:active=move || !favourite.get().unwrap_or_default()
                        href=user_article_href>
                        {move || route_user_id.get().map_or("Unknown User".to_string(), |id| id.to_string())}"'s Articles"
                    </a>
                </li>
                <li class="nav-item">
                    <a class="nav-link"
                        class:active=move || favourite.get().unwrap_or_default()
                        href=favourites_href>
                        "Favorited Articles"
                    </a>
                </li>
            </ul>
        </div>
    }
}

#[allow(clippy::redundant_closure)]
#[tracing::instrument]
#[component]
pub fn Profile(user_id: crate::auth::UserIdSignal) -> impl IntoView {
    let params = use_params_map();
    tracing::info!("Profile params : {:?} ", params);

    let route_user_id = create_memo(move |_| {
        params
            .with(|x| x.get("user_id").cloned())
            .and_then(|id_str| uuid::Uuid::parse_str(&id_str).ok())
    });

    let query = use_query_map();
    let favourite = create_memo(move |_| {
        query.with(|x| x.get("favourites").map(|_| true))
    });

    let articles = create_resource(
        move || (favourite.get(), route_user_id.get().unwrap_or_default()),
        move |(fav, user_id)| async move { profile_articles(user_id, fav).await },
    );

    view! {
        <Title text=move || format!("{}'s profile", route_user_id.get().map_or("Unknown User".to_string(), |id| id.to_string())) />
        <div class="profile-page">
            <UserInfo logged_user_id=user_id />

            <div class="container">
                <div class="row">
                    <div class="col-xs-12 col-md-10 offset-md-1">
                        <ProfileTabs
                            route_user_id=route_user_id
                            favourite=favourite
                        />

                        <Suspense
                            fallback=move || view! {
                                <div class="article-preview">
                                    "Loading articles..."
                                </div>
                            }
                        >
                            <ErrorBoundary
                                fallback=|_| view! {
                                    <div class="article-preview">
                                        "Error loading articles. Please try again later."
                                    </div>
                                }
                            >
                                <ArticlePreviewList user_id=user_id articles=articles />
                            </ErrorBoundary>
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn UserInfo(logged_user_id: crate::auth::UserIdSignal) -> impl IntoView {
    let params = use_params_map();

    let resource = create_resource(
        move || {
            params.with(|x| {
                x.get("user_id")
                    .cloned()
                    .and_then(|id| match uuid::Uuid::parse_str(&id) {
                        Ok(parsed_id) => Some(parsed_id),
                        Err(e) => {
                            tracing::error!("Failed to parse UUID: {:?}", e);
                            None
                        }
                    })
            })
        },
        move |user_id| async move {
            if let Some(valid_id) = user_id {
                user_profile(valid_id).await
            } else {
                tracing::error!("Invalid or missing UUID");
                Err(ServerFnError::ServerError(
                    "Invalid or missing UUID".to_string(),
                ))
            }
        },
    );

    view! {
        <div class="user-info">
            <div class="container">
                <div class="row">
                    <div class="col-xs-12 col-md-10 offset-md-1">
                    <Suspense
                        fallback=move || view!{<p>"Loading user profile"</p>}
                    >
                        <ErrorBoundary
                            fallback=|_| {
                                view!{<p>"There was a problem while fetching the user profile, try again later"</p>}
                            }
                        >
                            {move || {
                                resource.get().map(move |x| {
                                    x.map(move |u| {
                                        let image = u.user.image();
                                        let username = u.user.name();
                                        let bio = u.user.bio();
                                        let author_id = create_rw_signal(u.user.id()).read_only();

                                        view!{
                                            <img src=image class="user-img" />
                                            <h4>{username}</h4>
                                            <p>{bio.unwrap_or("No bio available".into())}</p>
                                            <ButtonFollow
                                                logged_user_id
                                                author_id
                                                following=u.following.unwrap_or_default()
                                            />
                                        }
                                    })
                                })
                            }}
                        </ErrorBoundary>
                    </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}