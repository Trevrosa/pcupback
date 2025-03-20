use pcupback::{Fetchable, Storable};
use sqlx::{Executor, FromRow, Sqlite, sqlite::SqliteQueryResult};

use super::public::{AppInfo, UserDebug};

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct DBAppInfo {
    pub user_id: u32,
    pub app_name: String,
    pub app_usage: u32,
    pub app_limit: u32,
}

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct DBUserDebug {
    pub user_id: u32,
    pub stored: String,
}

impl PartialEq<UserDebug> for DBUserDebug {
    fn eq(&self, other: &UserDebug) -> bool {
        self.stored == other.stored
    }
}

impl<'a> Storable<'a> for DBUserDebug {
    type DB = Sqlite;

    async fn store<E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: Executor<'a, Database = Self::DB>,
    {
        sqlx::query!(
            "INSERT INTO user_debug(user_id, stored) VALUES(?, ?)",
            self.user_id,
            self.stored
        )
        .execute(executor)
        .await
    }
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

    /// Create a [`DBAppInfo`] from an [`AppInfo`] by supplying a `user_id`.
    #[must_use]
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

impl<'a> Fetchable<'a, u32> for DBUserDebug {
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
        sqlx::query_as("SELECT * FROM user_debug WHERE user_id = ?")
            .bind(filter)
            .fetch_all(executor)
            .await
    }
}

#[cfg(test)]
mod tests {
    use pcupback::Storable;
    use sqlx::{Pool, Sqlite};

    use crate::routes::{auth::data::private::DBUser, sync::data::private::DBAppInfo};

    #[test]
    fn db_appinfo_eq() {
        let a = DBAppInfo::new_raw(1, "ddd", 2, 0);
        let b = DBAppInfo::new_raw(1, "d", 2, 0);
        let a_same = DBAppInfo::new_raw(1, "ddd", 2, 0);

        assert_eq!(a, a);
        assert_eq!(a, a_same);
        assert_ne!(a, b);
        assert_ne!(b, a_same);
    }

    #[sqlx::test]
    fn store_app_usage(db: Pool<Sqlite>) {
        DBUser::new_raw(1, "test", "pp").store(&db).await.unwrap();
        DBAppInfo::new_raw(1, "xdd", 12, 0)
            .store(&db)
            .await
            .unwrap();
    }
}
