use serde::{Deserialize, Serialize};
use sqlx::{Database, Encode, Executor, Type, sqlite::SqliteQueryResult};
use thiserror::Error;

/// A type that can be stored into a database of type [`Self::DB`].
#[allow(async_fn_in_trait)]
pub trait Storable<'a> {
    // associated Database type.
    // we use an associated type instead of a generic type to disallow multiple implementations on a single type.
    /// The database the implementor is [`Storable`] for.
    type DB: Database;

    /// Store `self` into the defined [`Self::DB`] database.
    ///
    /// # Errors
    ///
    /// See [`sqlx::Error`].
    async fn store<E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>;
}

/// A type that is fetchable and filterable by `F` into a database of type [`Self::DB`].
#[allow(async_fn_in_trait)]
pub trait Fetchable<'a>: Sized {
    /// The database the implementor is [`Storable`] for.
    type DB: Database;

    /// Fetch a `Self` from the [`Self::DB`] database, using `filter` to filter results.
    ///
    /// # Errors
    ///
    /// See [`sqlx::Error`].
    async fn fetch<E, F>(filter: F, executor: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
        F: Encode<'a, Self::DB> + Type<Self::DB> + 'a;
}

#[allow(unused)]
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum DBErrorKind {
    #[error("InsertError")]
    InsertError(String),
    #[error("SelectError")]
    SelectError(String),
}
