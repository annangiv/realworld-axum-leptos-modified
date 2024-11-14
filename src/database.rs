use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::OnceLock;

static DB: OnceLock<PgPool> = OnceLock::new();

#[tracing::instrument]
async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| sqlx::Error::Configuration(
            "DATABASE_URL environment variable not set".into()
        ))?;

    let pool = PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    sqlx::migrate!()
        .run(&pool)
        .await?;

    Ok(pool)
}

#[tracing::instrument]
pub async fn init_db() -> Result<(), sqlx::Error> {
    match create_pool().await {
        Ok(pool) => {
            if let Err(_) = DB.set(pool) {
                Err(sqlx::Error::Configuration(
                    "Failed to initialize database pool - already initialized".into()
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
    }
}

#[tracing::instrument]
pub fn get_db() -> &'static PgPool {
    DB.get().expect("Database not initialized. Call init_db() first")
}

pub async fn with_db<F, R>(f: F) -> Result<R, sqlx::Error>
where
    F: FnOnce(&PgPool) -> Result<R, sqlx::Error>,
{
    let pool = get_db();
    f(pool)
}

pub async fn with_transaction<F, R>(f: F) -> Result<R, sqlx::Error>
where
    F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<R, sqlx::Error>,
{
    let pool = get_db();
    let mut tx = pool.begin().await?;
    
    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_initialization() {
        // Ensure DATABASE_URL is set for tests
        std::env::set_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/test_db");
        
        // Test initialization
        let init_result = init_db().await;
        assert!(init_result.is_ok(), "Database initialization failed");

        // Test double initialization
        let second_init = init_db().await;
        assert!(second_init.is_err(), "Second initialization should fail");

        // Test get_db
        let db = get_db();
        assert!(db.acquire().await.is_ok(), "Could not acquire connection from pool");
    }
}