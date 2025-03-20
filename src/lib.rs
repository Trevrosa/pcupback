use serde::{Deserialize, Serialize};
use sqlx::{Database, Encode, Executor, Type, sqlite::SqliteQueryResult};
use thiserror::Error;

/// A type that can be stored into a database of type [`Self::DB`].
///
/// Only one implementation is allowed per type.
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

/// A type that is fetchable to [`Self`], filterable by `F` from a database of type [`Self::DB`].
///
/// Multiple implementations on the same type are allowed for different `F`.
///
/// Only one implementation with `F` and [`Self::DB`] is allowed per type.
#[allow(async_fn_in_trait)]
pub trait Fetchable<'a, F>: Sized
where
    F: Encode<'a, Self::DB> + Type<Self::DB> + 'a,
{
    /// The database the implementor is [`Fetchable`] for.
    type DB: Database;

    /// Fetch one Self from the [`Self::DB`] database, using `filter` to filter.
    ///
    /// # Errors
    ///
    /// See [`sqlx::Error`].
    async fn fetch_one<E>(filter: F, executor: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB> + Copy;

    /// Fetch all Self from the [`Self::DB`] database, using `filter` to filter.
    ///
    /// The default implementation creates a one-item [`Vec`] filled by [`Self::fetch_one`]
    ///
    /// # Errors
    ///
    /// See [`sqlx::Error`].
    async fn fetch_all<E>(filter: F, executor: E) -> Result<Vec<Self>, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB> + Copy,
    {
        Ok(vec![Self::fetch_one(filter, executor).await?])
    }
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum DBErrorKind {
    #[error("InsertError")]
    InsertError(String),
    #[error("SelectError")]
    SelectError(String),
    #[error("DeleteError")]
    DeleteError(String),
    #[error("OtherError")]
    OtherError(String),
}
