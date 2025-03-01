use argon2::{
    Argon2, PasswordHash, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use rocket_db_pools::sqlx::FromRow;
use sqlx::{Executor, Sqlite, sqlite::SqliteQueryResult};
use uuid::Uuid;

use crate::routes::auth::HashErrorKind::{self, CreateError};

/// A type that can be stored into a database of type [`Self::DB`].
pub trait Storable<'a> {
    // associated Database type.
    // we use an associated type instead of a generic type to disallow multiple implementations on a single type.
    type DB;

    /// Store `self` into the defined [`Self::DB`].
    ///
    /// # Errors
    ///
    /// See [`sqlx::Error`].
    async fn store<E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>;
}

#[derive(Debug, FromRow)]
pub struct DBUser {
    pub id: u32,
    pub username: String,
    pub password_hash: String,
}

impl DBUser {
    /// Create a new user to be stored in db.
    ///
    /// Hashes the given password.
    pub fn new(id: u32, username: &str, password: &str) -> Result<Self, HashErrorKind> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| CreateError(e.to_string()))?
            .to_string();

        Ok(Self {
            id,
            username: username.to_string(),
            password_hash,
        })
    }

    #[allow(unused)]
    pub fn with_hashed(id: u32, username: String, hashed: &PasswordHash) -> Self {
        let password_hash = hashed.to_string();

        Self {
            id,
            username,
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
        sqlx::query("INSERT INTO users(id, username, password_hash) VALUES(?, ?, ?)")
            .bind(self.id)
            .bind(self.username.clone())
            .bind(self.password_hash.clone())
            .execute(executor)
            .await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Sqlite, pool::Pool};

    use crate::routes::auth::data::private::{DBUser, Storable};

    use super::DBUserSession;

    #[sqlx::test]
    async fn store_session(db: Pool<Sqlite>) {
        let session = DBUserSession::generate(1);
        session.store(&db).await.unwrap();
    }

    #[test]
    fn user_creation() {
        DBUser::new(1, "test", "12345678").unwrap();
    }

    #[sqlx::test]
    async fn store_user(db: Pool<Sqlite>) {
        let user = DBUser::new(1, "test", "12345678").unwrap();
        user.store(&db).await.unwrap();
    }
}

#[derive(Debug, FromRow)]
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
