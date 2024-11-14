use leptos::*;
use leptos_router::*;

#[server(FollowAction, "/api")]
#[tracing::instrument]
pub async fn follow_action(other_user_id: uuid::Uuid) -> Result<bool, ServerFnError> {
    let Some(user_id) = crate::auth::get_user_id() else {
        return Err(ServerFnError::ServerError(
            "You need to be authenticated".into(),
        ));
    };
    toggle_follow(user_id, other_user_id).await.map_err(|x| {
        tracing::error!("problem while updating the database: {x:?}");
        ServerFnError::ServerError("error while updating the follow".into())
    })
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn toggle_follow(current: uuid::Uuid, other: uuid::Uuid) -> Result<bool, sqlx::Error> {
    let db = crate::database::get_db();
    match sqlx::query!(
        "SELECT * FROM Follows WHERE follower_id=$1 AND influencer_id=$2",
        current,
        other
    )
    .fetch_one(db)
    .await
    {
        Ok(_) => sqlx::query!(
            "DELETE FROM Follows WHERE follower_id=$1 AND influencer_id=$2",
            current,
            other
        )
        .execute(db)
        .await
        .map(|_| false),
        Err(sqlx::error::Error::RowNotFound) => sqlx::query!(
            "INSERT INTO Follows(follower_id, influencer_id) VALUES ($1, $2)",
            current,
            other
        )
        .execute(db)
        .await
        .map(|_| true),
        Err(x) => Err(x),
    }
}

#[component]
pub fn ButtonFollow(
    logged_user_id: crate::auth::UserIdSignal,
    author_id: ReadSignal<Option<uuid::Uuid>>,
    following: bool,
) -> impl IntoView {
    let follow = create_server_action::<FollowAction>();
    let result_call = follow.value();
    let is_loading = follow.pending();
    
    let follow_cond = move || {
        if let Some(x) = result_call.get() {
            match x {
                Ok(x) => x,
                Err(err) => {
                    tracing::error!("problem while following {err:?}");
                    following
                }
            }
        } else {
            following
        }
    };

    view! {
        <Show
            when=move || {
                let logged_id = logged_user_id.get().unwrap_or_default();
                let author = author_id.get().unwrap_or_default();
                logged_id != author
            }
            fallback=|| ()
        >
            <ActionForm action=follow class="inline pull-xs-right">
                <input
                    type="hidden"
                    name="other_user_id"
                    value=move || author_id.get().unwrap_or_default().to_string()
                />
                <button 
                    type="submit" 
                    class="btn btn-sm btn-outline-secondary"
                    disabled=is_loading
                >
                    <Show
                        when=move || is_loading.get()
                        fallback=move || {
                            view! {
                                <Show
                                    when=follow_cond
                                    fallback=|| view!{<i class="ion-plus-round"></i>" Follow "}
                                >
                                    <i class="ion-close-round"></i>" Unfollow "
                                </Show>
                            }
                        }
                    >
                        "Loading..."
                    </Show>
                    {" "}
                    {move || author_id.get().unwrap_or_default().to_string()}
                </button>
            </ActionForm>
        </Show>
    }
}

#[server(FavAction, "/api")]
#[tracing::instrument]
pub async fn fav_action(article_id: uuid::Uuid) -> Result<bool, ServerFnError> {
    tracing::info!("Article for Favorite {:?} ", article_id);

    let Some(user_id) = crate::auth::get_user_id() else {
        return Err(ServerFnError::ServerError(
            "You need to be authenticated".into(),
        ));
    };
    toggle_fav(article_id, user_id).await.map_err(|x| {
        tracing::error!("problem while updating the database: {x:?}");
        ServerFnError::ServerError("error while updating the favorite".into())
    })
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn toggle_fav(article_id: uuid::Uuid, user_id: uuid::Uuid) -> Result<bool, sqlx::Error> {
    let db = crate::database::get_db();
    match sqlx::query!(
        "SELECT * FROM FavArticles WHERE article_id=$1 AND user_id=$2",
        article_id,
        user_id
    )
    .fetch_one(db)
    .await
    {
        Ok(_) => sqlx::query!(
            "DELETE FROM FavArticles WHERE article_id=$1 AND user_id=$2",
            article_id,
            user_id
        )
        .execute(db)
        .await
        .map(|_| false),
        Err(sqlx::error::Error::RowNotFound) => sqlx::query!(
            "INSERT INTO FavArticles(article_id, user_id) VALUES ($1, $2)",
            article_id,
            user_id
        )
        .execute(db)
        .await
        .map(|_| true),
        Err(x) => Err(x),
    }
}

#[component]
pub fn ButtonFav(
    user_id: crate::auth::UserIdSignal,
    article: super::article_preview::ArticleSignal,
) -> impl IntoView {
    let make_fav = create_server_action::<FavAction>();
    let result_make_fav = make_fav.value();
    let is_loading = make_fav.pending();
    
    let fav_count = move || {
        if let Some(x) = result_make_fav.get() {
            match x {
                Ok(result) => {
                    article.update(move |x| {
                        x.fav = !x.fav;
                        x.favorites_count =
                            (x.favorites_count + if result { 1 } else { -1 }).max(0);
                    });
                }
                Err(err) => {
                    tracing::error!("problem while favoriting {err:?}");
                }
            }
        }
        article.with(|x| x.favorites_count)
    };

    view! {
        <Show
            when=move || user_id.with(Option::is_some)
            fallback=move || view!{
                <button class="btn btn-sm btn-outline-primary pull-xs-right">
                    <i class="ion-heart"></i>
                    <span class="counter">" ("{fav_count}")"</span>
                </button>
            }
        >
            <ActionForm action=make_fav class="inline pull-xs-right">
                <input 
                    type="hidden" 
                    name="article_id" 
                    value=move || article.with(|x| x.id.to_string()) 
                />
                <button 
                    type="submit" 
                    class="btn btn-sm btn-outline-primary"
                    disabled=is_loading
                >
                    <Show
                        when=move || is_loading.get()
                        fallback=move || {
                            view! {
                                <Show
                                    when=move || article.with(|x| x.fav)
                                    fallback=move || view!{<i class="ion-heart"></i>" Fav "}
                                >
                                    <i class="ion-heart-broken"></i>" Unfav "
                                </Show>
                            }
                        }
                    >
                        "Loading..."
                    </Show>
                    <span class="counter">"("{fav_count}")"</span>
                </button>
            </ActionForm>
        </Show>
    }
}