use pcupback::{DBErrorKind, Fetchable};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Sqlite};
use thiserror::Error;

use super::private::DBAppInfo;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UserData {
    pub app_usage: Vec<AppInfo>,
}

impl<'a> Fetchable<'a, u32> for UserData {
    type DB = Sqlite;

    /// Aggregates associated data in a [`UserData`], converts from in-db values.
    async fn fetch_one<E>(filter: u32, executor: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        let app_usage: Vec<AppInfo> = DBAppInfo::fetch_all(filter, executor)
            .await?
            .into_iter()
            .map(|a: DBAppInfo| a.into())
            .collect();

        Ok(Self { app_usage })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AppInfo {
    pub name: String,
    pub(super) usage: u32,
    pub(super) limit: u32,
}

impl AppInfo {
    #[cfg(test)]
    pub fn new(name: impl Into<String>, usage: u32, limit: u32) -> Self {
        Self {
            name: name.into(),
            usage,
            limit,
        }
    }
}

impl From<DBAppInfo> for AppInfo {
    fn from(value: DBAppInfo) -> Self {
        Self {
            name: value.app_name,
            usage: value.app_usage,
            limit: value.app_limit,
        }
    }
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum SyncError {
    #[error("InvalidSession")]
    InvalidSession,
    #[error("DBError")]
    DBError(#[from] DBErrorKind),
}

#[cfg(test)]
mod tests {
    use pcupback::{Fetchable, Storable};
    use sqlx::{Pool, Sqlite};

    use crate::routes::{
        auth::data::private::DBUser,
        sync::data::{private::DBAppInfo, public::UserData},
    };

    #[sqlx::test]
    fn fetch_user_data(db: Pool<Sqlite>) {
        DBUser::new_raw(1, "test", "pp").store(&db).await.unwrap();

        let app_info = DBAppInfo::new_raw(1, "xddapp", 1, 0);
        app_info.store(&db).await.unwrap();

        let data = UserData::fetch_one(1, &db).await.unwrap();
        assert_eq!(data.app_usage.len(), 1);
        assert_eq!(data.app_usage[0].name, app_info.app_name);
        assert_eq!(data.app_usage[0].usage, app_info.app_usage);
    }
}
