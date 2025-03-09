use chrono::{DateTime, TimeDelta, Utc};
use pcupback::Storable;
use sqlx::{Executor, Sqlite};

use crate::routes::auth::data::{private::DBUserSession, public::UserSession};

pub(crate) const SESSION_TIMEOUT: TimeDelta = TimeDelta::days(1);

/// Return `true` if `dt` is more than [`SESSION_TIMEOUT`] ago, else, return `false`.
pub(crate) fn session_timeout(dt: DateTime<Utc>) -> bool {
    Utc::now() - dt > SESSION_TIMEOUT
}

/// Check if `session` is timed out. If it is, generate and store a new one.
pub(crate) async fn validate_session<'a>(
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
pub(crate) async fn generate_store_session(
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

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, Utc};
    use pcupback::Storable;
    use sqlx::{Pool, Sqlite};

    use crate::{routes::auth::data::private::DBUser, util::auth::SESSION_TIMEOUT};

    #[sqlx::test]
    async fn generate_store_session(db: Pool<Sqlite>) {
        // no such user id `1`
        super::generate_store_session(&db, 1).await.unwrap_err();

        // generate the user, now ok.
        DBUser::new_raw(1, "ppk1", "12").store(&db).await.unwrap();
        super::generate_store_session(&db, 1).await.unwrap();
    }

    #[test]
    fn session_timeout() {
        // not timeout
        assert!(!super::session_timeout(Utc::now()));
        assert!(!super::session_timeout(
            Utc::now() - SESSION_TIMEOUT + TimeDelta::seconds(1)
        ));
        // yes timeout
        assert!(super::session_timeout(Utc::now() - SESSION_TIMEOUT));
        assert!(super::session_timeout(
            Utc::now() - SESSION_TIMEOUT - TimeDelta::seconds(1)
        ));
    }
}
