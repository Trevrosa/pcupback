/// The authentication endpoint.
///
/// # Receives:
/// An username and password. Or, a [`AuthRequest`].
///
/// # Returns:
/// In Json, the requested user's session if ok, else an [`AuthError`]. Or, a [`Json<Result<UserSession, AuthError>>`]
pub mod auth;

// TODO: docs
pub mod delete_account;

// TODO: reset password

/// The user data synchronization endpoint.
///
/// # Receives:
/// The `session_id` of the requested user **and** the client's optional local [`UserData`]. (a [`Option<UserData>`])
///
/// # Returns:
/// In Json, the final stored user data if ok, else an [`SyncError`]. Or, a [`Json<Result<UserData, SyncError>>`]
pub mod sync;
