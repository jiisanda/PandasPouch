use sqlx::postgres::{PgPool, PgPoolOptions};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Database {
            pool,
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT value FROM cache WHERE key = $1"
        )
            .bind(key)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn put(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO cache (key, value) VALUES ($1, $2)\
            ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        )
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn create_table_if_not_exists(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS cache (\
                    key TEXT PRIMARY KEY,\
                    value TEXT NOT NULL\
            )",
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
