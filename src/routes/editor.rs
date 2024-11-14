use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    UpdateError,
    Successful(String),
}

#[derive(Debug)]
struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: Vec<String>,
}

const TITLE_MIN_LENGTH: usize = 4;
const DESCRIPTION_MIN_LENGTH: usize = 4;
const BODY_MIN_LENGTH: usize = 10;

#[cfg(feature = "ssr")]
#[tracing::instrument]
fn validate_article(
    title: String,
    description: String,
    body: String,
    tag_list: String,
) -> Result<ArticleUpdate, String> {
    if title.len() < TITLE_MIN_LENGTH {
        return Err(format!("Title must be at least {TITLE_MIN_LENGTH} characters"));
    }

    if description.len() < DESCRIPTION_MIN_LENGTH {
        return Err(format!("Description must be at least {DESCRIPTION_MIN_LENGTH} characters"));
    }

    if body.len() < BODY_MIN_LENGTH {
        return Err(format!("Body must be at least {BODY_MIN_LENGTH} characters"));
    }

    let tag_list = tag_list
        .trim()
        .split_ascii_whitespace()
        .filter(|x| !x.is_empty())
        .map(String::from)
        .collect();

    Ok(ArticleUpdate {
        title,
        description,
        body,
        tag_list,
    })
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn update_article(
    author_id: uuid::Uuid,
    slug: String,
    article: ArticleUpdate,
) -> Result<String, sqlx::Error> {
    let mut transaction = crate::database::get_db().begin().await?;

    let (rows_affected, new_slug) = if !slug.is_empty() {
        // Update existing article
        let rows = sqlx::query!(
            "UPDATE Articles SET title=$1, description=$2, body=$3 WHERE slug=$4 and author_id=$5",
            article.title,
            article.description,
            article.body,
            slug,
            author_id,
        )
        .execute(transaction.as_mut())
        .await?
        .rows_affected();
        (rows, slug)
    } else {
        // Create new article
        let new_slug = article
            .title
            .chars()
            .map(|c| {
                let c = c.to_ascii_lowercase();
                if c == ' ' { '-' } else { c }
            })
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect::<String>();

        let rows = sqlx::query!(
            "INSERT INTO Articles(slug, title, description, body, tags, author_id) 
             VALUES ($1, $2, $3, $4, $5, $6)",
            new_slug,
            article.title,
            article.description,
            article.body,
            &article.tag_list,
            author_id
        )
        .execute(transaction.as_mut())
        .await?
        .rows_affected();
        (rows, new_slug)
    };

    if rows_affected != 1 {
        tracing::error!("Expected 1 row affected, got {}", rows_affected);
        return Err(sqlx::Error::RowNotFound);
    }

    transaction.commit().await?;
    Ok(new_slug)
}

#[server(EditorAction, "/api")]
#[tracing::instrument]
pub async fn editor_action(
    title: String,
    description: String,
    body: String,
    tag_list: String,
    slug: String,
) -> Result<EditorResponse, ServerFnError> {
    let Some(author_id) = crate::auth::get_user_id() else {
        leptos_axum::redirect("/login");
        return Ok(EditorResponse::ValidationError("Authentication required".to_string()));
    };

    match validate_article(title, description, body, tag_list) {
        Ok(article) => match update_article(author_id, slug, article).await {
            Ok(new_slug) => {
                leptos_axum::redirect(&format!("/article/{new_slug}"));
                Ok(EditorResponse::Successful(new_slug))
            }
            Err(err) => {
                tracing::error!("Article update failed: {}", err);
                Ok(EditorResponse::UpdateError)
            }
        },
        Err(validation_error) => Ok(EditorResponse::ValidationError(validation_error)),
    }
}

#[component]
pub fn Editor() -> impl IntoView {
    let editor_server_action = create_server_action::<EditorAction>();
    let result = editor_server_action.value();
    
    let error = move || {
        result.with(|x| {
            x.as_ref().map_or(true, |result| {
                result.is_err() || !matches!(result, Ok(EditorResponse::Successful(_)))
            })
        })
    };

    let params = use_params_map();
    let article_res = create_resource(
        move || params.get(),
        |params| async move {
            if let Some(slug) = params.get("slug") {
                super::get_article(slug.to_string()).await
            } else {
                Ok(super::ArticleResult::default())
            }
        },
    );

    view! {
        <Title text="Editor"/>
        <div class="editor-page">
            <div class="container page">
                <div class="row">
                    <p class="text-xs-center"
                        class:text-success=move || !error()
                        class:error-messages=error
                    >
                        <strong>
                            {move || result.with(|result| match result {
                                Some(Ok(EditorResponse::ValidationError(msg))) => {
                                    format!("Validation error: {msg}")
                                }
                                Some(Ok(EditorResponse::UpdateError)) => {
                                    "Failed to update article. Please try again later.".to_string()
                                }
                                Some(Ok(EditorResponse::Successful(_))) => String::new(),
                                Some(Err(err)) => format!("Unexpected error: {err}"),
                                None => String::new(),
                            })}
                        </strong>
                    </p>

                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <ActionForm action=editor_server_action>
                            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                                <ErrorBoundary fallback=|_| {
                                    view! { <p class="error-messages text-xs-center">"Something went wrong."</p> }
                                }>
                                    {move || article_res.get().map(|result| result.map(|article| {
                                        view! {
                                            <fieldset>
                                                <fieldset class="form-group">
                                                    <input 
                                                        name="title"
                                                        type="text"
                                                        class="form-control form-control-lg"
                                                        minlength=TITLE_MIN_LENGTH
                                                        placeholder="Article Title"
                                                        value=article.article.title
                                                    />
                                                </fieldset>
                                                <fieldset class="form-group">
                                                    <input 
                                                        name="description"
                                                        type="text"
                                                        class="form-control"
                                                        minlength=DESCRIPTION_MIN_LENGTH
                                                        placeholder="What's this article about?"
                                                        value=article.article.description
                                                    />
                                                </fieldset>
                                                <fieldset class="form-group">
                                                    <textarea 
                                                        name="body"
                                                        class="form-control"
                                                        rows="8"
                                                        placeholder="Write your article (in markdown)"
                                                        minlength=BODY_MIN_LENGTH
                                                        prop:value=article.article.body.unwrap_or_default()
                                                    ></textarea>
                                                </fieldset>
                                                <fieldset class="form-group">
                                                    <input 
                                                        name="tag_list"
                                                        type="text"
                                                        class="form-control"
                                                        placeholder="Enter tags (space separated)"
                                                        value=article.article.tag_list.join(" ")
                                                    />
                                                </fieldset>
                                                <input 
                                                    name="slug"
                                                    type="hidden"
                                                    value=article.article.slug
                                                />
                                                <button 
                                                    class="btn btn-lg pull-xs-right btn-primary"
                                                    type="submit"
                                                >
                                                    "Publish Article"
                                                </button>
                                            </fieldset>
                                        }
                                    }))}
                                </ErrorBoundary>
                            </Suspense>
                        </ActionForm>
                    </div>
                </div>
            </div>
        </div>
    }
}