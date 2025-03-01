use serde::{Deserialize, Serialize};

use super::private::DBUserSession;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct User {
//     pub name: String,
//     pub id: u32,
// }

// impl From<DBUser> for User {
//     fn from(value: DBUser) -> Self {
//         Self {
//             name: value.username,
//             id: value.id,
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: u32,
    pub id: String,
}

impl From<DBUserSession> for UserSession {
    fn from(value: DBUserSession) -> Self {
        Self {
            user_id: value.user_id,
            id: value.id,
        }
    }
}
