use log::{debug, error, info, warn};
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use sqlx::{Acquire, PgPool};
use std::{fmt::Debug, iter, time::Duration};
use tokio::time::sleep;
use sha2::{Sha256, Digest};


#[derive(Deserialize, Debug)]
struct Article {
    id: i32,
    user: User,
}

#[derive(Deserialize, Debug)]
struct ArticleDetail {
    title: String,
    slug: String,
    tags: Vec<String>,
    description: String,
    body_html: Option<String>,
    cover_image: Option<String>,
    reading_time_minutes: Option<i32>,
    user: UserSummary,
}

#[derive(Deserialize, Debug)]
struct UserSummary {
    user_id: i32,
}

#[derive(Deserialize, Debug)]
struct User {
    username: String,
    name: String,
    summary: Option<String>,
    profile_image: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = Client::new();

    let database_url = std::env::var("DATABASE_URL").map_err(|e| {
        eprintln!("Error: DATABASE_URL environment variable not set or accessible.");
        e
    })?;

    let pool = PgPool::connect(&database_url).await?;

    for page in 1..=100 {
        let dev_url = format!(
            "https://dev.to/api/articles/latest?page={}&per_page=50",
            page
        );

        debug!("Fetching articles from: {}", &dev_url);

        let response = client
            .get(&dev_url)
            .header("User-Agent", "Mozilla/5.0 (compatible; MyRustApp/1.0)")
            .send()
            .await?;

        let articles: Vec<Article> = match response.json().await {
            Ok(articles) => articles,
            Err(e) => {
                error!("Failed to parse JSON for articles: {}", e);
                continue;
            }
        };

        for article in articles {
            let article_url = format!("https://dev.to/api/articles/{}", article.id);
            let detailed_article: ArticleDetail = match client
                .get(&article_url)
                .header("User-Agent", "Mozilla/5.0 (compatible; MyRustApp/1.0)")
                .send()
                .await?
                .json()
                .await
            {
                Ok(article) => article,
                Err(e) => {
                    error!("Failed to fetch article details: {}", e);
                    continue;
                }
            };

            let author_id = match create_user_helper(
                article.user.name.clone(),
                format!("{}@example.com", article.user.username),
                "defaultpassword".to_string(),
                "".to_string(), // If bio is not provided
                article.user.profile_image.unwrap_or_default(),
                &pool,
            )
            .await
            {
                Ok(author_id) => author_id,
                Err(e) => {
                    warn!("User creation failed: {}", e);
                    continue;
                }
            };

            if let Err(e) = create_article_helper(
                detailed_article.title.clone(),
                detailed_article.slug.clone(),
                detailed_article.description.clone(),
                detailed_article.body_html.clone().unwrap_or_default(),
                author_id,
                detailed_article.tags.clone(),
                detailed_article.cover_image.unwrap_or_default(),
                detailed_article.reading_time_minutes.unwrap_or(0),
                &pool,
            )
            .await
            {
                warn!("Article creation failed: {}", e);
            }

            // Sleep between article detail requests to avoid rate-limiting
            sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}

async fn create_user_helper(
    name: String,
    email: String,
    password: String,
    bio: String,
    image: String,
    pool: &PgPool,
) -> Result<uuid::Uuid, Box<dyn std::error::Error>> {
    debug!("Creating user with name: {}, email: {}", name, email);

    let username = generate_username(&name);
    let email_hash = hash_email(&email);

    let mut transaction = pool.begin().await?;
    let connection = transaction.acquire().await?;

    let user_id = match sqlx::query!(
        "INSERT INTO Users (name, username, email, password, bio, image, email_hash) VALUES ($1, $2, $3, crypt($4, gen_salt('bf')), $5, $6, $7) RETURNING id",
        name,
        username,
        email,
        password,
        bio,
        image,
        email_hash
    )
    .fetch_one(connection)
    .await
    {
        Ok(record) => {
            transaction.commit().await?;
            info!("User {} created successfully", username);
            record.id
        }
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            warn!(
                "User with email {} already exists, skipping creation.",
                email
            );
            // Fetch the existing user ID from the database
            let existing_user_id = sqlx::query!(
                "SELECT id FROM Users WHERE email = $1",
                email
            )
            .fetch_one(pool)
            .await?
            .id;
            existing_user_id
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            return Err(e.into());
        }
    };

    Ok(user_id)
}

async fn create_article_helper(
    title: String,
    slug: String,
    description: String,
    body: String,
    author_id: uuid::Uuid,
    tags: Vec<String>,
    cover_image: String,
    reading_time: i32,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut transaction = pool.begin().await?;
    let connection = transaction.acquire().await?;

    match sqlx::query!(
        "INSERT INTO Articles (title, description, slug, body, author_id, tags, cover_image, reading_time, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())",
        title,
        description,
        slug,
        body,
        author_id,
        &tags,
        cover_image,
        reading_time
    )
    .execute(connection)
    .await
    {
        Ok(_) => {
            transaction.commit().await?;
            info!("Article {} created successfully", slug);
        }
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            warn!("Article with slug {} already exists, skipping creation.", slug);
        }
        Err(e) => {
            error!("Failed to create article: {}", e);
            return Err(e.into());
        }
    };

    Ok(())
}

fn generate_username(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();

    let random_string: String = iter::repeat(())
        .map(|()| rand::thread_rng().sample(Alphanumeric))
        .take(7)
        .map(char::from)
        .collect();

    match words.as_slice() {
        [first, last, ..] => format!(
            "{}_{}_{}",
            first.to_lowercase(),
            last.to_lowercase(),
            random_string
        ),
        [first] => format!("{}_{}", first.to_lowercase(), random_string),
        _ => random_string,
    }
}

fn hash_email(email: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(email);
    format!("{:x}", hasher.finalize()) // Returns the hash as a hex string
}
