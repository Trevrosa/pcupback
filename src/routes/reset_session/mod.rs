use pcupback::{DBErrorKind, Fetchable};
use rocket::{State, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use thiserror::Error;

use crate::{
    routes::auth::data::private::DBUserSession,
    util::{auth::generate_store_session, db::ToExecutor},
};

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

#[put("/auth/reset_session/<session_id>")]
pub async fn reset_session(
    state: &State<Pool<Sqlite>>,
    session_id: &str,
) -> Json<ResetSessionResult> {
    use DBErrorKind::InsertError;
    use ResetSessionError::{DBError, InvalidSession};

    let db = state.to_db();

    let session = DBUserSession::fetch_one(session_id, db).await;

    let Ok(session) = session else {
        // no such session.
        tracing::info!("session was invalid");
        return Json(Err(InvalidSession));
    };

    let user_id = session.user_id;

    let new_session = generate_store_session(db, user_id)
        .await
        .map_err(|err| DBError(InsertError(err.to_string())));

    Json(new_session)
}
