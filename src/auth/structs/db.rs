use chrono::Utc;
use rocket_db_pools::sqlx::FromRow;
use sqlx::{Executor, Sqlite, sqlite::SqliteQueryResult};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct DBUser {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, FromRow)]
pub struct DBUserSession {
    pub user_id: u32,
    pub session_id: String,
    pub last_set: u32,
}

pub trait Storable {
    /// Store `self` into a [`Sqlite`] database.
    ///
    /// # Errors
    ///
    /// Bubbles errors from query.execute.
    async fn store<'a, E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Sqlite>;
}

impl DBUserSession {
    /// Create a new user session to be stored in database.
    ///
    /// `last_set` is Now. session_id is Uuid::new_v4
    pub fn new(user_id: u32) -> Self {
        Self {
            user_id,
            session_id: Uuid::new_v4().to_string(),
            last_set: Utc::now().timestamp() as u32,
        }
    }
}

impl Storable for DBUserSession {
    async fn store<'a, E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        sqlx::query(
            "INSERT OR REPLACE INTO sessions(user_id, session_id, last_set) VALUES(?, ?, ?)",
        )
        .bind(self.user_id)
        .bind(&self.session_id)
        .bind(self.last_set)
        .execute(executor)
        .await
    }
}
