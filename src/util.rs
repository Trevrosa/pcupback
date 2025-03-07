use chrono::{DateTime, Utc};
use pcupback::Storable;
use sqlx::{Executor, Sqlite};

use crate::routes::auth::{
    SESSION_TIMEOUT,
    data::{private::DBUserSession, public::UserSession},
};

pub fn session_timeout(dt: DateTime<Utc>) -> bool {
    Utc::now().time() - dt.time() > SESSION_TIMEOUT
}

/// Check if `session` is timed out. If it is, generate and store a new one.
pub async fn validate_session<'a>(
    executor: impl Executor<'a, Database = Sqlite>,
    session: Result<DBUserSession, sqlx::Error>,
    new_id: u32,
) -> Result<UserSession, sqlx::Error> {
    if let Ok(session) = session {
        // if `last_set` was more than `SESSION_TIMEOUT` ago, we create a new session.
        let session_last_set = session.last_set_datetime();

        if let Some(session_last_set) = session_last_set {
            if session_timeout(session_last_set) {
                tracing::info!("session timed out, generating new one");
                generate_store_session(executor, new_id).await
            } else {
                // session is ok, return it
                Ok(session.into())
            }
        } else {
            generate_store_session(executor, new_id).await
        }
    } else {
        tracing::warn!("no session, generating one");
        generate_store_session(executor, new_id).await
    }
}

// store a session, returning Ok(session)
pub async fn generate_store_session(
    executor: impl Executor<'_, Database = Sqlite>,
    user_id: u32,
) -> Result<UserSession, sqlx::Error> {
    let session = DBUserSession::generate(user_id);
    match session.store(executor).await {
        // stored session successfully, return
        Ok(_) => Ok(session.into()),
        Err(err) => {
            tracing::error!("failed to store session: {err:?}");
            Err(err)
        }
    }
}
