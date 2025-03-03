use argon2::{
    Argon2, PasswordHash, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use pcupback::{Fetchable, Storable};
use sqlx::{Executor, FromRow, Sqlite, sqlite::SqliteQueryResult};
use uuid::Uuid;

use super::public::HashErrorKind::{self, CreateError};

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct DBUser {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
}

impl DBUser {
    /// Create a new user to be stored in db.
    ///
    /// Hashes the given password.
    pub fn new(
        id: u32,
        username: impl Into<String>,
        password: impl AsRef<str>,
    ) -> Result<Self, HashErrorKind> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_ref().as_bytes(), &salt)
            .map_err(|e| CreateError(e.to_string()))?
            .to_string();

        Ok(Self {
            id,
            username: username.into(),
            password_hash,
        })
    }

    pub fn new_store(username: impl Into<String>, password: impl AsRef<str>) -> Result<Self, HashErrorKind> {
        Self::new(0, username, password)
    }

    #[allow(unused)]
    pub fn with_hashed(id: u32, username: impl Into<String>, hashed: &PasswordHash) -> Self {
        let password_hash = hashed.to_string();

        Self {
            id,
            username: username.into(),
            password_hash,
        }
    }
}

impl<'a> Storable<'a> for DBUser {
    type DB = Sqlite;

    async fn store<E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        sqlx::query("INSERT INTO users(username, password_hash) VALUES(?, ?)")
            // .bind(self.id)
            .bind(self.username.clone())
            .bind(self.password_hash.clone())
            .execute(executor)
            .await
    }
}

impl<'a> Fetchable<'a, &'a str> for DBUser {
    type DB = Sqlite;

    /// username filter
    async fn fetch_one<E>(filter: &str, executor: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        sqlx::query_as("SELECT * FROM users WHERE username = ?")
            .bind(filter)
            .fetch_one(executor)
            .await
    }
}

impl<'a> Fetchable<'a, u32> for DBUser {
    type DB = Sqlite;

    /// user id filter
    async fn fetch_one<E>(filter: u32, executor: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(filter)
            .fetch_one(executor)
            .await
    }
}

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct DBUserSession {
    pub user_id: u32,
    pub id: String,
    /// Stored as seconds since the unix epoch.
    pub last_set: i64,
}

impl DBUserSession {
    /// Generate a new user session to be stored in database.
    ///
    /// `last_set` is [`Utc::now`]. `session_id` is [`Uuid::new_v4`]
    pub fn generate(user_id: u32) -> Self {
        Self {
            user_id,
            id: Uuid::new_v4().to_string(),
            last_set: Utc::now().timestamp(),
        }
    }

    /// Parse the session's `last_set` to a [`chrono::DateTime`], or [`None`] if parsing fails.
    pub fn last_set_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.last_set, 0)
    }
}

// make user session storable for `Sqlite` databases
impl<'a> Storable<'a> for DBUserSession {
    type DB = Sqlite;

    async fn store<E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        sqlx::query("INSERT OR REPLACE INTO sessions(user_id, id, last_set) VALUES(?, ?, ?)")
            .bind(self.user_id)
            .bind(&self.id)
            .bind(self.last_set)
            .execute(executor)
            .await
    }
}

#[cfg(test)]
mod tests {
    use pcupback::Fetchable;
    use sqlx::{Error, Sqlite, pool::Pool};

    use crate::routes::auth::data::private::{DBUser, Storable};

    use super::DBUserSession;

    #[sqlx::test]
    async fn store_session(db: Pool<Sqlite>) {
        DBUserSession::generate(1).store(&db).await.unwrap();
    }

    #[test]
    fn user_creation() {
        DBUser::new(1, "test", "12345678").unwrap();
    }

    #[sqlx::test]
    async fn store_user(db: Pool<Sqlite>) {
        DBUser::new(1, "test", "12345678")
            .unwrap()
            .store(&db)
            .await
            .unwrap();
    }

    #[sqlx::test]
    async fn fetch_user_id(db: Pool<Sqlite>) {
        let stored = DBUser::new(1, "test", "12345678").unwrap();
        stored.store(&db).await.unwrap();
        let fetched = DBUser::fetch_one(1, &db).await.unwrap();

        assert_eq!(fetched, stored)
    }

    #[sqlx::test]
    async fn fetch_user_name(db: Pool<Sqlite>) {
        let stored = DBUser::new(1, "test", "12345678").unwrap();
        stored.store(&db).await.unwrap();
        let fetched = DBUser::fetch_one("test", &db).await.unwrap();

        assert_eq!(fetched, stored);

        let not_exists = DBUser::fetch_one("testdd", &db).await;
        assert!(matches!(not_exists.unwrap_err(), Error::RowNotFound))
    }
}
