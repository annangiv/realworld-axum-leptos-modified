use leptos::*;
use leptos_router::*;
use super::buttons::{ButtonFav, ButtonFollow};

pub type ArticleSignal = RwSignal<crate::models::Article>;
type ArticlesType<S, T = Result<Vec<crate::models::Article>, ServerFnError>> = Resource<S, T>;

#[component]
pub fn ArticlePreviewList<S: 'static + std::clone::Clone>(
    user_id: crate::auth::UserIdSignal,
    articles: ArticlesType<S>,
) -> impl IntoView {
    let loading = articles.loading();
    
    let articles_view = move || {
        articles.with(move |x| {
            x.clone().map(move |res| {
                let articles = res.unwrap_or_default();
                if articles.is_empty() {
                    view! {
                        <div class="article-preview">
                            "No articles are here... yet."
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <For
                            each=move || articles.clone().into_iter().enumerate()
                            key=|(_, article)| article.slug.clone()
                            children=move |(_, article)| {
                                let article = create_rw_signal(article);
                                view! {
                                    <ArticlePreview article=article user_id=user_id />
                                }
                            }
                        />
                    }.into_view()
                }
            })
        })
    };

    view! {
        <Suspense fallback=move || {
            view! {
                <div class="article-preview">
                    <p class="text-xs-center">
                        {move || if loading.get() {
                            "Loading articles..."
                        } else {
                            "Preparing articles..."
                        }}
                    </p>
                </div>
            }
        }>
            <ErrorBoundary fallback=|_errors| {
                view! { 
                    <div class="article-preview">
                        <p class="error-messages text-xs-center">
                            "Error loading articles: "
                        </p>
                    </div>
                }
            }>
                {articles_view}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ArticlePreview(user_id: crate::auth::UserIdSignal, article: ArticleSignal) -> impl IntoView {
    view! {
        <div class="article-preview">
            <ArticleMeta user_id=user_id article=article is_preview=true />
            <A href=move || format!("/article/{}", article.with(|x| x.slug.clone())) class="preview-link">
                <h1>{move || article.with(|x| x.title.to_string())}</h1>
                <p>{move || article.with(|x| x.description.to_string())}</p>
                <span class="btn">"Read more..."</span>
                <TagList article/>
            </A>
        </div>
    }
}

#[component]
fn TagList(article: ArticleSignal) -> impl IntoView {
    view! {
        <Show
            when=move || article.with(|x| !x.tag_list.is_empty())
            fallback=|| view! {<span class="no-tags">"No tags"</span>}
        >
            <ul class="tag-list">
                <i class="ion-pound"></i>
                <For
                    each=move || article.with(|x| x.tag_list.clone())
                    key=|tag| tag.clone()
                    children=move |tag| {
                        view! {
                            <li class="tag-default tag-pill tag-outline">
                                <A href=format!("/?tag={tag}")>{tag}</A>
                            </li>
                        }
                    }
                />
            </ul>
        </Show>
    }
}

#[component]
pub fn ArticleMeta(
    user_id: crate::auth::UserIdSignal,
    article: ArticleSignal,
    is_preview: bool,
) -> impl IntoView {
    let editor_ref = move || format!("/editor/{}", article.with(|x| x.slug.to_string()));
    let profile_ref = move || format!("/profile/{}", article.with(|x| x.author.user_id.to_string()));
    let delete_a = create_server_action::<DeleteArticleAction>();
    
    let is_author = move || {
        user_id.get().map_or(false, |id| id == article.with(|x| x.author.user_id))
    };

    view! {
        <div class="article-meta">
            <A href=profile_ref>
                <img 
                    src=move || article.with(|x| x.author.image.clone().unwrap_or_default())
                    alt=move || format!("{}'s avatar", article.with(|x| x.author.name.clone()))
                />
            </A>
            <div class="info">
                <A href=profile_ref class="author">
                    {move || article.with(|x| x.author.name.to_string())}
                </A>
                <span class="date">{move || article.with(|x| x.created_at.to_string())}</span>
            </div>
            <Show
                when=move || is_preview
                fallback=move || {
                    view! {
                        <Show
                            when=is_author
                            fallback=move || {
                                let following = article.with(|x| x.author.following);
                                let author_id = create_rw_signal(Some(article.with(|x| x.author.user_id))).read_only();

                                view! {
                                    <Show when=move || user_id.with(Option::is_some) fallback=|| ()>
                                        <ButtonFav user_id=user_id article=article />
                                        <ButtonFollow
                                            logged_user_id=user_id
                                            author_id=author_id
                                            following=following
                                        />
                                    </Show>
                                }
                            }
                        >
                            <A class="btn btn-sm btn-outline-secondary" href=editor_ref>
                                <i class="ion-compose"></i>
                                " Edit article"
                            </A>
                            <ActionForm action=delete_a class="inline">
                                <input 
                                    type="hidden" 
                                    name="slug" 
                                    value=move || article.with(|x| x.slug.to_string()) 
                                />
                                <button 
                                    type="submit" 
                                    class="btn btn-sm btn-outline-danger"
                                    disabled=move || delete_a.pending().get()
                                >
                                    <i class="ion-trash-a"></i>
                                    {move || if delete_a.pending().get() {
                                        " Deleting..."
                                    } else {
                                        " Delete article"
                                    }}
                                </button>
                            </ActionForm>
                        </Show>
                    }
                }
            >
                <ButtonFav user_id=user_id article=article />
            </Show>
        </div>
    }
}

#[server(DeleteArticleAction, "/api")]
#[tracing::instrument]
pub async fn delete_article(slug: String) -> Result<(), ServerFnError> {
    let Some(logged_user_id) = crate::auth::get_user_id() else {
        return Err(ServerFnError::ServerError("You must be logged in".into()));
    };
    let redirect_profile = format!("/profile/{logged_user_id}");

    crate::models::Article::delete(slug, logged_user_id)
        .await
        .map(move |_| leptos_axum::redirect(&redirect_profile))
        .map_err(|x| {
            let err = format!("Error while deleting an article: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not delete the article, try again later".into())
        })
}