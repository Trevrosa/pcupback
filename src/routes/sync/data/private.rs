use pcupback::Storable;
use sqlx::{FromRow, Sqlite, sqlite::SqliteQueryResult};

#[derive(Debug, FromRow)]
pub struct DBAppInfo {
    pub user_id: u32,
    pub app_name: String,
    pub app_usage: u32,
    pub app_limit: u32,
}

impl DBAppInfo {
    pub fn new(user_id: u32, app_name: impl Into<String>, app_usage: u32, app_limit: u32) -> Self {
        Self {
            user_id,
            app_name: app_name.into(),
            app_usage,
            app_limit,
        }
    }

    // pub fn duration(&self) -> Duration {
    //     Duration::from_secs(self.app_usage as u64)
    // }
}

impl<'a> Storable<'a> for DBAppInfo {
    type DB = Sqlite;

    async fn store<E>(&self, executor: E) -> Result<SqliteQueryResult, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = Self::DB>,
    {
        sqlx::query(
            "INSERT INTO app_info(user_id, app_name, app_usage, app_limit) VALUES(?, ?, ?, ?)",
        )
        .bind(self.user_id)
        .bind(&self.app_name)
        .bind(self.app_usage)
        .bind(self.app_limit)
        .execute(executor)
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
        DBAppInfo::new(1, "xdd", 12, 0).store(&db).await.unwrap();
    }
}
