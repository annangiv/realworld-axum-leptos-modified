use super::UserPreview;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Article {
    pub id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub description: String,
    pub created_at: String,
    pub favorites_count: i64,
    pub tag_list: Vec<String>,
    pub author: UserPreview,
    pub fav: bool,
}

impl Article {
    #[cfg(feature = "ssr")]
    pub async fn for_home_page(
        page: i64,
        amount: i64,
        tag: String,
        my_feed: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let user_id = crate::auth::get_user_id();
        sqlx::query!(
            "
            SELECT 
                a.id,
                a.slug,
                a.title,
                a.description,
                a.created_at,
                a.tags AS tag_list,
                (SELECT COUNT(*) FROM FavArticles WHERE article_id = a.id) AS favorites_count,
                u.id AS author_id,
                u.name,
                u.username,
                u.image,
                EXISTS(SELECT 1 FROM FavArticles WHERE article_id = a.id AND user_id = $5) AS fav,
                EXISTS(SELECT 1 FROM Follows WHERE follower_id = $5 AND influencer_id = u.id) AS following
            FROM Articles AS a
            JOIN Users AS u ON a.author_id = u.id
            WHERE
                ($3 = '' OR $3 = ANY(a.tags))
                AND
                (NOT $4 OR u.id IN (SELECT influencer_id FROM Follows WHERE follower_id = $5))
            ORDER BY a.created_at DESC
            LIMIT $1 OFFSET $2",
            amount,
            page * amount,
            tag,
            my_feed,
            user_id,
        )
        .map(|x| Self {
            id: x.id,
            slug: x.slug,
            title: x.title,
            body: None, // no need
            fav: x.fav.unwrap_or_default(),
            description: x.description,
            created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
            favorites_count: x.favorites_count.unwrap_or_default(),
            author: UserPreview {
                user_id: x.author_id,
                name: x.name,
                username: x.username,
                image: x.image,
                following: x.following.unwrap_or_default(),
            },
            tag_list: x.tag_list.unwrap_or_default(),
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn for_user_profile(
        user_id: uuid::Uuid,
        favourites: bool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let logged_user_id = crate::auth::get_user_id();
        sqlx::query!(
            "
            SELECT 
                a.id,
                a.slug,
                a.title,
                a.description,
                a.created_at,
                u.id as author_id,
                u.username,
                u.name,
                u.image,
                (SELECT COUNT(*) FROM FavArticles WHERE article_id = a.id) AS favorites_count,
                EXISTS(SELECT 1 FROM FavArticles WHERE article_id = a.id AND user_id = $2) AS fav,
                EXISTS(SELECT 1 FROM Follows WHERE follower_id = $2 AND influencer_id = a.author_id) AS following,
                a.tags AS tag_list
            FROM Articles AS a
            JOIN Users AS u ON u.id = a.author_id
            WHERE
                CASE WHEN $3 THEN
                    EXISTS(SELECT fa.article_id FROM FavArticles AS fa WHERE fa.article_id = a.id AND fa.user_id = $1)
                ELSE a.author_id = $1
                END",
            user_id,
            logged_user_id,
            favourites,
        )
        .map(|x| Self {
            id: x.id,
            slug: x.slug,
            title: x.title,
            body: None, // no need
            fav: x.fav.unwrap_or_default(),
            description: x.description,
            created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
            favorites_count: x.favorites_count.unwrap_or_default(),
            tag_list: x.tag_list.unwrap_or_default(),
            author: UserPreview {
                user_id: x.author_id,
                name: x.name,
                username: x.username,
                image: x.image,
                following: x.following.unwrap_or_default(),
            },
        })
        .fetch_all(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn for_article(slug: String) -> Result<Self, sqlx::Error> {
        let user_id = crate::auth::get_user_id();

        sqlx::query!(
            "
            SELECT
                a.id, a.slug, a.title, a.description, a.body, a.created_at,
                a.tags AS tag_list,
                (SELECT COUNT(*) FROM FavArticles WHERE article_id = a.id) AS fav_count,
                u.id AS author_id,
                u.username,
                u.image,
                u.name,
                EXISTS(SELECT 1 FROM FavArticles WHERE article_id = a.id AND user_id = $2) AS fav,
                EXISTS(SELECT 1 FROM Follows WHERE follower_id = $2 AND influencer_id = a.author_id) AS following
            FROM Articles a
            JOIN Users u ON a.author_id = u.id
            WHERE a.slug = $1
            ",
            slug,
            user_id,
        )
        .map(|x| Self {
            id: x.id,
            slug: x.slug,
            title: x.title,
            description: x.description,
            body: Some(x.body),
            tag_list: x.tag_list.unwrap_or_default(),
            favorites_count: x.fav_count.unwrap_or_default(),
            created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
            fav: x.fav.unwrap_or_default(),
            author: UserPreview {
                user_id: x.author_id,
                name: x.name,
                username: x.username,
                image: x.image,
                following: x.following.unwrap_or_default(),
            },
        })
        .fetch_one(crate::database::get_db())
        .await
    }

    #[cfg(feature = "ssr")]
    pub async fn delete(
        slug: String,
        author_id: uuid::Uuid,
    ) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "DELETE FROM Articles WHERE slug=$1 AND author_id=$2",
            slug,
            author_id
        )
        .execute(crate::database::get_db())
        .await
    }
}
