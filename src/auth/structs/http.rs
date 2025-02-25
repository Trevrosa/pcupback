use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user: User,
    pub session: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}