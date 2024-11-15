use crate::components::ArticlePreviewList;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[server(HomeAction, "/api", "GetJson")]
async fn home_articles(
    page: u32,
    amount: u32,
    tag: String,
    my_feed: bool,
) -> Result<Vec<crate::models::Article>, ServerFnError> {
    let page = i64::from(page);
    let amount = i64::from(amount);

    Ok(
        crate::models::Article::for_home_page(page, amount, tag, my_feed)
            .await
            .map_err(|x| {
                tracing::error!("problem while fetching home articles: {x:?}");
                ServerFnError::ServerError("Problem while fetching home articles".into())
            })?,
    )
}

#[server(GetTagsAction, "/api", "GetJson")]
async fn get_tags() -> Result<Vec<String>, ServerFnError> {
    sqlx::query!(
        "
        SELECT tag
        FROM unnest(
            (SELECT array_agg(tag) FROM Articles, unnest(tags) AS tag WHERE tag <> '')
        ) AS tag
        GROUP BY tag
        ORDER BY COUNT(*) DESC
        LIMIT 10
        "
    )
    .fetch_all(crate::database::get_db())
    .await
    .map(|rows| {
        rows.into_iter()
            .filter_map(|row| row.tag)
            .collect()
    })
    .map_err(|x| {
        tracing::error!("problem while fetching tags: {x:?}");
        ServerFnError::ServerError("Problem while fetching tags".into())
    })
}

#[component]
pub fn HomePage(user_id: crate::auth::UserIdSignal) -> impl IntoView {
    let pagination = use_query::<crate::models::Pagination>();

    let articles = create_resource(
        move || pagination.get().unwrap_or_default(),
        move |pagination| async move {
            tracing::debug!("making another request: {pagination:?}");
            home_articles(
                pagination.get_page(),
                pagination.get_amount(),
                pagination.get_tag().to_string(),
                pagination.get_my_feed(),
            )
            .await
        },
    );

    view! {
        <Title text="Home"/>

        <div class="home-page">
            <div class="banner">
                <div class="container">
                    <h1 class="logo-font">"thedeveloper"</h1>
                    <p>"Empowering Developers, One Line of Code at a Time."</p>
                </div>
            </div>

            <div class="container page">
                <div class="row">
                    <div class="col-md-9">
                        <div class="feed-toggle">
                            <ul class="nav nav-pills outline-active">
                                <li class="nav-item">
                                    <a class="nav-link"
                                        class:active=move || !pagination.with(|x| x.as_ref().map(crate::models::Pagination::get_my_feed).unwrap_or_default())
                                        href=move || pagination.get().unwrap_or_default().reset_page().set_my_feed(false).to_string()>
                                        "Global Feed"
                                    </a>
                                </li>
                                {move || {
                                    if user_id.with(|name| name.is_some()) {
                                        view! {
                                            <li class="nav-item">
                                                <a href=move || {
                                                    if user_id.with(Option::is_some)
                                                        && !pagination.with(|x| {
                                                            x.as_ref()
                                                                .map(crate::models::Pagination::get_my_feed)
                                                                .unwrap_or_default()
                                                        })
                                                    {
                                                        pagination
                                                            .get()
                                                            .unwrap_or_default()
                                                            .reset_page()
                                                            .set_my_feed(true)
                                                            .to_string()
                                                    } else {
                                                        String::new()
                                                    }
                                                }
                                                class=move || {
                                                    format!(
                                                        "nav-link {}",
                                                        if user_id.with(Option::is_none) {
                                                            "disabled"
                                                        } else if pagination.with(|x| x
                                                            .as_ref()
                                                            .map(crate::models::Pagination::get_my_feed)
                                                            .unwrap_or_default())
                                                        {
                                                            "active"
                                                        } else {
                                                            ""
                                                        }
                                                    )
                                                }>
                                                    "Your Feed"
                                                </a>
                                            </li>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }
                                }}
                                <li class="nav-item pull-xs-right">
                                    <div style="display: inline-block;">
                                        "Articles to display | "
                                        <a href=move || pagination.get().unwrap_or_default().reset_page().set_amount(1).to_string() class="btn btn-primary">"1"</a>
                                        <a href=move || pagination.get().unwrap_or_default().reset_page().set_amount(20).to_string() class="btn btn-primary">"20"</a>
                                        <a href=move || pagination.get().unwrap_or_default().reset_page().set_amount(50).to_string() class="btn btn-primary">"50"</a>
                                    </div>
                                </li>
                            </ul>
                        </div>

                        <ArticlePreviewList articles user_id/>
                    </div>

                    <div class="col-md-3">
                        <div class="sidebar">
                            <h4>"Popular Tags"</h4>
                            <TagList pagination/>
                        </div>
                    </div>

                    <ul class="pagination">
                        <Show
                            when=move || {pagination.with(|x| x.as_ref().map(crate::models::Pagination::get_page).unwrap_or_default()) > 0}
                            fallback=|| ()
                        >
                            <li class="page-item">
                                <a class="btn btn-primary" href=move || pagination.get().unwrap_or_default().previous_page().to_string()>
                                    "<< Previous page"
                                </a>
                            </li>
                        </Show>
                        <Suspense fallback=|| ()>
                            <Show
                                when=move || {
                                    let n_articles = articles.with(|x| x.as_ref().map_or(0, |y| y.as_ref().map(Vec::len).unwrap_or_default()));
                                    let page_size = pagination.with(|x| x.as_ref().map(crate::models::Pagination::get_amount).unwrap_or_default()) as usize;
                                    n_articles > 0 && n_articles >= page_size
                                }
                                fallback=|| ()
                            >
                                <li class="page-item">
                                    <a class="btn btn-primary" href=move || pagination.get().unwrap_or_default().next_page().to_string()>
                                        "Next page >>"
                                    </a>
                                </li>
                            </Show>
                        </Suspense>
                    </ul>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TagList(pagination: Memo<Result<crate::models::Pagination, ParamsError>>) -> impl IntoView {
    let tag_list = create_resource(|| (), |_| async { get_tags().await });

    view! {
        <div class="tag-list">
            <Suspense fallback=move || view! {<p>"Loading Tags"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! { <p class="error-messages text-xs-center">"Something went wrong."</p>}
                }>
                    {move || {
                        tag_list.get().map(|ts| {
                            ts.map(|tags| {
                                let tag_elected = pagination.with(|x| {
                                    x.as_ref()
                                        .ok()
                                        .map(crate::models::Pagination::get_tag)
                                        .unwrap_or_default()
                                        .to_string()
                                });
                                
                                view! {
                                    <For
                                        each=move || tags.clone().into_iter().enumerate()
                                        key=|(i, _)| *i
                                        children=move |(_, t): (usize, String)| {
                                            let t2 = t.to_string();
                                            let same = t2 == tag_elected;
                                            view!{
                                                <a class="tag-pill tag-default" 
                                                   class:tag-primary=same
                                                   href=move || pagination
                                                       .get()
                                                       .ok()
                                                       .unwrap_or_default()
                                                       .set_tag(if same {""} else {&t2})
                                                       .to_string()>
                                                    {t}
                                                </a>
                                            }
                                        }
                                    />
                                }
                            })
                        })
                    }}
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}