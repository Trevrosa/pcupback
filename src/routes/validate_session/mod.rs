#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use pcupback::Fetchable;
use rocket::{State, get, serde::json::Json};
use sqlx::{Pool, Sqlite};

use crate::util::{auth::session_timeout, db::PoolStateExt};

use super::auth::data::private::DBUserSession;

#[inline]
fn not_timed_out(dt: DateTime<Utc>) -> bool {
    !session_timeout(dt)
}

#[get("/auth/validate_session/<session_id>")]
pub async fn validate_session(state: &State<Pool<Sqlite>>, session_id: &str) -> Json<bool> {
    let db = state.to_db();

    let valid = DBUserSession::fetch_one(session_id, db)
        .await
        .is_ok_and(|f| f.id == session_id && f.last_set_datetime().is_some_and(not_timed_out));
    Json(valid)
}
