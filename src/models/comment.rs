#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Comment {
    pub id: uuid::Uuid,
    pub article_id: uuid::Uuid,
    pub user_id: uuid::Uuid, // Changed from username to user_id (UUID)
    pub body: String,
    pub created_at: String,
    pub user_image: Option<String>,
    pub username: String,
    pub name: String,
}

impl Comment {
    #[cfg(feature = "ssr")]
    pub async fn insert(
        article_id: uuid::Uuid,
        user_id: uuid::Uuid, // Changed to user_id
        body: String,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO Comments(article_id, user_id, body) VALUES ($1, $2, $3)",
            article_id,
            user_id,
            body
        )
        .execute(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn get_all(article_id: uuid::Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query!(
            "
        SELECT c.*, u.image, u.username, u.name FROM Comments as c
            JOIN Users as u ON u.id = c.user_id
        WHERE c.article_id = $1
        ORDER BY c.created_at",
            article_id
        )
        .map(|x| Self {
            id: x.id,
            article_id: x.article_id,
            user_id: x.user_id, // Using user_id instead of username
            body: x.body,
            created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
            user_image: x.image,
            username: x.username,
            name: x.name,
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn delete(
        id: uuid::Uuid,
        user_id: uuid::Uuid, // Changed to user_id
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "DELETE FROM Comments WHERE id=$1 AND user_id=$2",
            id,
            user_id
        )
        .execute(crate::database::get_db())
        .await
    }
}
