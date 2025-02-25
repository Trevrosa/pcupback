use rocket_db_pools::sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct DBUser {
    pub id: u32,
    pub username: String,
    pub hashed_password: String,
}
