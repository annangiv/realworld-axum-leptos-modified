use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::ArticleMeta;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct ArticleResult {
    pub(super) article: crate::models::Article,
    pub(super) logged_user: Option<crate::models::User>,
}

#[server(GetArticleAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_article(slug: String) -> Result<ArticleResult, ServerFnError> {
    Ok(ArticleResult {
        article: crate::models::Article::for_article(slug)
            .await
            .map_err(|x| {
                let err = format!("Error while getting user_profile articles: {x:?}");
                tracing::error!("{err}");
                ServerFnError::ServerError("Could not retrieve articles, try again later".into())
            })?,
        logged_user: crate::auth::current_user().await.ok(),
    })
}

#[tracing::instrument]
#[component]
pub fn Article(user_id: crate::auth::UserIdSignal) -> impl IntoView {
    let params = use_params_map();
    let article = create_resource(
        move || params.get().get("slug").cloned().unwrap_or_default(),
        |slug| async { get_article(slug).await },
    );

    let title = create_rw_signal(String::from("Loading"));

    view! {
        <Title text=move || title.get()/>

        <Suspense fallback=move || view! { 
            <div class="article-page">
                <div class="container">
                    <div class="article-content">
                        <p>"Loading Article..."</p>
                    </div>
                </div>
            </div>
        }>
            <ErrorBoundary fallback=|errors| {
                view! { 
                    <div class="article-page">
                        <div class="container">
                            <div class="article-content">
                                <p class="error-messages text-xs-center">
                                    "Error loading article. Please try again later."
                                </p>
                                <pre class="error-detail">{format!("{:?}", errors)}</pre>
                            </div>
                        </div>
                    </div>
                }
            }>
                {move || {
                    article.get().map(move |x| {
                        x.map(move |article_result| {
                            title.set(article_result.article.slug.to_string());
                            view! {
                                <ArticlePage user_id result=article_result />
                            }
                        })
                    })
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ArticlePage(user_id: crate::auth::UserIdSignal, result: ArticleResult) -> impl IntoView {
    let article_signal = create_rw_signal(result.article.clone());
    let user_signal = create_rw_signal(result.logged_user);
    let tag_list = result.article.tag_list.clone();

    view! {
        <article class="article-page">
            <header class="banner">
                <div class="container">
                    <h1>{result.article.title}</h1>
                    <ArticleMeta user_id article=article_signal is_preview=false />
                </div>
            </header>

            <div class="container page">
                <div class="row article-content">
                    <div class="col-md-12">
                        <div 
                            class="article-body"
                            inner_html={result.article.body}
                        ></div>
                    </div>
                </div>

                <ul class="tag-list" role="list" aria-label="Article tags">
                    <For
                        each=move || tag_list.clone().into_iter().enumerate()
                        key=|(i, _)| *i
                        children=|(_, tag)| {
                            view!{
                                <li class="tag-default tag-pill tag-outline">
                                    <A href=format!("/?tag={}", tag)>{tag}</A>
                                </li>
                            }
                        }
                    />
                </ul>

                <hr />

                <div class="article-actions">
                    <div class="row" style="justify-content: center;">
                        <ArticleMeta user_id article=article_signal is_preview=false />
                    </div>
                </div>

                <div class="row">
                    <CommentSection user_id article=article_signal user=user_signal />
                </div>
            </div>
        </article>
    }
}

#[server(PostCommentAction, "/api")]
#[tracing::instrument]
pub async fn post_comment(article_id: uuid::Uuid, body: String) -> Result<(), ServerFnError> {
    if body.trim().is_empty() {
        return Err(ServerFnError::ServerError("Comment cannot be empty".into()));
    }

    let Some(user_id) = crate::auth::get_user_id() else {
        return Err(ServerFnError::ServerError("You must be logged in to comment".into()));
    };

    crate::models::Comment::insert(article_id, user_id, body.trim().to_string())
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post comment. Please try again later.".into())
        })
}

#[server(GetCommentsAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_comments(
    article_id: uuid::Uuid,
) -> Result<Vec<crate::models::Comment>, ServerFnError> {
    crate::models::Comment::get_all(article_id)
        .await
        .map_err(|x| {
            let err = format!("Error fetching comments: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not load comments. Please try again later.".into())
        })
}

#[server(DeleteCommentsAction, "/api")]
#[tracing::instrument]
pub async fn delete_comment(id: uuid::Uuid) -> Result<(), ServerFnError> {
    let Some(user_id) = crate::auth::get_user_id() else {
        return Err(ServerFnError::ServerError("You must be logged in".into()));
    };

    crate::models::Comment::delete(id, user_id)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error deleting comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not delete comment. Please try again later.".into())
        })
}

#[component]
fn CommentSection(
    user_id: crate::auth::UserIdSignal,
    article: crate::components::ArticleSignal,
    user: RwSignal<Option<crate::models::User>>,
) -> impl IntoView {
    let comments_action = create_server_action::<PostCommentAction>();
    let result = comments_action.version();
    let reset_comment = create_rw_signal("");
    let is_submitting = comments_action.pending();
    
    let comments = create_resource(
        move || (result.get(), article.with(|a| a.id)),
        move |(_, article_id)| async move {
            reset_comment.set("");
            get_comments(article_id).await
        },
    );

    view! {
        <section class="col-xs-12 col-md-8 offset-md-2">
            <Show 
                when=move || user_id.with(Option::is_some) 
                fallback=|| view! {
                    <p class="text-center">
                        <A href="/login">"Sign in"</A>
                        " or "
                        <A href="/register">"sign up"</A>
                        " to add comments on this article"
                    </p>
                }
            >
                <ActionForm action=comments_action class="card comment-form">
                    <input 
                        name="article_id" 
                        type="hidden" 
                        value=move || article.with(|x| x.id.to_string()) 
                    />
                    <div class="card-block">
                        <textarea 
                            name="body"
                            prop:value=move || reset_comment.get()
                            class="form-control"
                            placeholder="Write a comment..."
                            rows="3"
                            disabled=move || is_submitting.get()
                        ></textarea>
                    </div>
                    <div class="card-footer">
                        <img 
                            src=move || user.with(|x| x.as_ref().map(crate::models::User::image).unwrap_or_default())
                            class="comment-author-img" 
                            alt="Your profile picture"
                        />
                        <button 
                            class="btn btn-sm btn-primary"
                            type="submit"
                            disabled=move || is_submitting.get()
                        >
                            {move || if is_submitting.get() { "Posting..." } else { "Post Comment" }}
                        </button>
                    </div>
                </ActionForm>
            </Show>

            <Suspense fallback=move || view! {
                <p class="text-center">"Loading comments..."</p>
            }>
                <ErrorBoundary fallback=|errors| {
                    view! { 
                        <p class="error-messages text-xs-center">
                            "Error loading comments. Please try again later."
                            <pre class="error-detail">{format!("{:?}", errors)}</pre>
                        </p>
                    }
                }>
                    {move || comments.get().map(move |x| x.map(move |c| {
                        view! {
                            <For 
                                each=move || c.clone().into_iter().enumerate()
                                key=|(i, _)| *i
                                children=move |(_, comment)| {
                                    let comment = create_rw_signal(comment);
                                    view!{<Comment user_id comment comments />}
                                }
                            />
                        }
                    }))}
                </ErrorBoundary>
            </Suspense>
        </section>
    }
}

#[component]
fn Comment<T: 'static + Clone, S: 'static>(
    user_id: crate::auth::UserIdSignal,
    comment: RwSignal<crate::models::Comment>,
    comments: Resource<T, S>,
) -> impl IntoView {
    let user_link = move || format!("/profile/{}", comment.with(|x| x.user_id.to_string()));
    let user_image = move || comment.with(|x| x.user_image.clone().unwrap_or_default());
    let delete_c = create_server_action::<DeleteCommentsAction>();
    let delete_result = delete_c.value();
    let show_delete_confirm = create_rw_signal(false);
    let is_deleting = delete_c.pending();

    create_effect(move |_| {
        if let Some(Ok(())) = delete_result.get() {
            tracing::info!("comment deleted!");
            show_delete_confirm.set(false);
            comments.refetch();
        }
    });

    view! {
        <div class="card">
            <div class="card-block">
                <p class="card-text">{move || comment.with(|x| x.body.to_string())}</p>
            </div>
            <div class="card-footer">
                <A href=user_link class="comment-author">
                    <img 
                        src=user_image 
                        class="comment-author-img"
                        alt="Commenter's profile picture"
                    />
                </A>
                " "
                <A href=user_link class="comment-author">
                    {move || comment.with(|x| x.user_id.to_string())}
                </A>
                <span class="date-posted" title=move || comment.with(|x| x.created_at.to_string())>
                    {move || comment.with(|x| {
                        // You might want to add a date formatting utility here
                        x.created_at.to_string()
                    })}
                </span>
                <Show
                    when=move || {user_id.get().unwrap_or_default() == comment.with(|x| x.user_id)}
                    fallback=|| ()
                >
                    {move || {
                        if show_delete_confirm.get() {
                            view! {
                                <div class="delete-confirm">
                                    <span>"Delete this comment?"</span>
                                    <ActionForm action=delete_c class="comment-author">
                                        <input 
                                            type="hidden" 
                                            name="id" 
                                            value=move || comment.with(|x| x.id.to_string()) 
                                        />
                                        <button 
                                            class="btn btn-sm btn-danger"
                                            type="submit"
                                            disabled=move || is_deleting.get()
                                        >
                                            {move || if is_deleting.get() { "Deleting..." } else { "Yes" }}
                                        </button>
                                    </ActionForm>
                                    <button 
                                        class="btn btn-sm"
                                        on:click=move |_| show_delete_confirm.set(false)
                                        disabled=move || is_deleting.get()
                                    >
                                        "No"
                                    </button>
                                </div>
                            }
                        } else {
                            view! {
                                <div class="delete-confirm">  // Wrapping div added here
                                    <button 
                                        class="btn btn-sm"
                                        on:click=move |_| show_delete_confirm.set(true)
                                    >
                                        "Delete"
                                    </button>
                                </div>
                            }
                        }
                    }}
                </Show>
            </div>
        </div>
    }
}