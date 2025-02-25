use serde::{Deserialize, Serialize};

use super::db::{DBUser};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: u32,
}

impl From<DBUser> for User {
    fn from(value: DBUser) -> Self {
        Self {
            name: value.username,
            id: value.id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user: User,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}
