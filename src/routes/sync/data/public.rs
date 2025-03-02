use std::time::Duration;

use pcupback::{DBErrorKind, Fetchable};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, Sqlite, Type};
use thiserror::Error;

use super::private::DBAppInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub app_usage: Vec<AppInfo>,
}

impl<'a> Fetchable<'a> for UserData {
    type DB = Sqlite;

    async fn fetch<E, F>(filter: F, executor: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
        F: Encode<'a, Self::DB> + Type<Self::DB> + 'a,
    {
        let app_usage: Vec<AppInfo> = sqlx::query_as("SELECT * FROM app_info WHERE user_id = ?")
            .bind(filter)
            .fetch_all(executor)
            .await?
            .into_iter()
            .map(|a: DBAppInfo| a.into())
            .collect();

        Ok(Self { app_usage })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    usage: u32,
}

impl AppInfo {
    pub fn usage(&self) -> Duration {
        Duration::from_secs(self.usage as u64)
    }
}

impl From<DBAppInfo> for AppInfo {
    fn from(value: DBAppInfo) -> Self {
        Self {
            name: value.app_name,
            usage: value.app_usage,
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

    use crate::routes::sync::data::{private::DBAppInfo, public::UserData};

    #[sqlx::test]
    fn fetch_user_data(db: Pool<Sqlite>) {
        let app_info = DBAppInfo::new(1, "xddapp", 1, 0);
        app_info.store(&db).await.unwrap();

        let data = UserData::fetch(1, &db).await.unwrap();
        assert_eq!(data.app_usage.len(), 1);
        assert_eq!(data.app_usage[0].name, app_info.app_name);
        assert_eq!(data.app_usage[0].usage, app_info.app_usage);
    }
}
