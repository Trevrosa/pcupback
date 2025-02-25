/// Data structs regarding authorization to be shared in requests
mod structs;

use argon2::password_hash::{SaltString, rand_core::OsRng};
use rocket::{post, serde::json::Json, Responder, State};
use rocket_db_pools::{Connection, sqlx};
use structs::{
    db::DBUser,
    http::{AuthRequest, UserSession},
};
use thiserror::Error;
use tracing::{Level, span};

use crate::{ARGON2, db::Db};

#[derive(Debug, Responder)]
enum AuthenticationError {
    // #[response(status = 403)]
    // InvalidPassword(PasswordErrorKind),
    #[response(status = 501)]
    FailedToHash,
}

#[derive(Error, Debug)]
enum PasswordErrorKind {
    #[error("not enough characters (min. 6)")]
    NotEnoughChars,
}

#[post("/auth", data = "<request>")]
async fn authenticate(
    mut db: Connection<Db>,
    request: Json<AuthRequest>,
) -> Result<Json<UserSession>, AuthenticationError> {
    use AuthenticationError::*;
    use PasswordErrorKind::*;

    let my_span = span!(Level::INFO, "authenticate");
    let _enter = my_span.enter();

    let existing_user: Option<DBUser> = sqlx::query_as("SELECT * FROM users WHERE id = {}")
        .bind(&request.username)
        .fetch_one(&mut **db)
        .await
        .ok();

    if existing_user.is_some() {
        let salt = SaltString::generate(&mut OsRng);
        let mut request_hashed_password = Vec::new();
        ARGON2
            .hash_password_into(
                request.password.as_ref(),
                salt.as_str().as_ref(),
                &mut request_hashed_password,
            )
            .map_err(|_| FailedToHash)?;
        // let request_hashed_password =
    }

    todo!()
}
