/// Data structs regarding authorization to be shared in requests 
mod structs;

use rocket::{post, serde::json::Json};
use rocket_db_pools::{sqlx, Connection};
use structs::AuthRequest;

use crate::db::Db;

#[post("/auth", data = "<request>")]
async fn authenticate(mut db: Connection<Db>, request: Json<AuthRequest>) -> Option<Json<AuthRequest>> {
    let existing_user: Result<User, _> = sqlx::query_as("SELECT * FROM users WHERE id = {}")
        .bind(&request.username)
        .fetch_one(&mut **db)
        .await;

    match existing_user {
        Ok(user) => {
            let user = user
        }
    }
            

    todo!()
}
