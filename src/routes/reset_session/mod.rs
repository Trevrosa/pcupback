use pcupback::DBErrorKind;
use rocket::{State, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use thiserror::Error;

use crate::util::generate_store_session;

use super::auth::data::public::UserSession;

#[cfg(test)]
mod tests;

// TODO: move the auth error to its own enum?
type ResetSessionResult = Result<UserSession, ResetSessionError>;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ResetSessionError {
    #[error("InvalidSession")]
    InvalidSession,
    #[error("DBError")]
    DBError(#[from] DBErrorKind),
}

#[put("/auth/reset_sesh/<session_id>")]
pub async fn reset_session(db: &State<Pool<Sqlite>>, session_id: &str) -> Json<ResetSessionResult> {
    use DBErrorKind::InsertError;
    use ResetSessionError::{DBError, InvalidSession};

    // FIXME:
    // TODO: test this works
    let db = &**db;

    let user_id = sqlx::query_as("SELECT user_id FROM sessions WHERE id = ?")
        .bind(session_id)
        .fetch_optional(db)
        .await
        .map(|o| o.map(|v: (u32,)| v.0));

    let Ok(Some(user_id)) = user_id else {
        // no such session.
        tracing::info!("session was invalid");
        return Json(Err(InvalidSession));
    };

    let new_session = generate_store_session(db, user_id)
        .await
        .map_err(|err| DBError(InsertError(err.to_string())));

    Json(new_session)
}
