use rocket_db_pools::sqlx::{self, FromRow};

#[derive(Debug, FromRow)]
pub struct DBUser {
    pub name: String
}
