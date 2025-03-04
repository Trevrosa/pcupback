use pcupback::{Fetchable, Storable};
use sqlx::{Executor, FromRow, Sqlite, sqlite::SqliteQueryResult};

use super::public::AppInfo;

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct DBAppInfo {
    pub user_id: u32,
    pub app_name: String,
    pub app_usage: u32,
    pub app_limit: u32,
}

impl DBAppInfo {
    #[cfg(test)]
    pub fn new_raw(
        user_id: u32,
        app_name: impl Into<String>,
        app_usage: u32,
        app_limit: u32,
    ) -> Self {
        Self {
            user_id,
            app_name: app_name.into(),
            app_usage,
            app_limit,
        }
    }

    pub fn with_app_info(user_id: u32, app_info: AppInfo) -> Self {
        Self {
            user_id,
            app_name: app_info.name,
            app_usage: app_info.usage,
            app_limit: app_info.limit,
        }
    }

    // pub fn duration(&self) -> Duration {
    //     Duration::from_secs(self.app_usage as u64)
    // }
}

impl PartialEq<AppInfo> for DBAppInfo {
    fn eq(&self, other: &AppInfo) -> bool {
        self.app_name == other.name && self.app_limit == other.limit
    }
}

impl<'a> Storable<'a> for DBAppInfo {
    type DB = Sqlite;

    async fn store<E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        sqlx::query!(
            "INSERT INTO app_info(user_id, app_name, app_usage, app_limit) VALUES(?, ?, ?, ?)",
            self.user_id,
            self.app_name,
            self.app_usage,
            self.app_limit
        )
        .execute(executor)
        .await
    }
}

impl<'a> Fetchable<'a, u32> for DBAppInfo {
    type DB = Sqlite;

    async fn fetch_one<E>(_filter: u32, _executor: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        unimplemented!()
    }

    async fn fetch_all<E>(filter: u32, executor: E) -> Result<Vec<Self>, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        sqlx::query_as("SELECT * FROM app_info WHERE user_id = ?")
            .bind(filter)
            .fetch_all(executor)
            .await
    }
}

#[cfg(test)]
mod tests {
    use pcupback::Storable;
    use sqlx::{Pool, Sqlite};

    use crate::routes::sync::data::private::DBAppInfo;

    #[sqlx::test]
    fn store_app_usage(db: Pool<Sqlite>) {
        DBAppInfo::new_raw(1, "xdd", 12, 0)
            .store(&db)
            .await
            .unwrap();
    }
}
