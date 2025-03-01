/// The authentication endpoint.
///
/// # Receives:
/// An username and password. Or, a [`AuthRequest`].
///
/// # Returns:
/// In Json, the requested user's session if ok, else an [`AuthError`]. Or, a [`Json<Result<UserSession, AuthError>>`]
pub mod auth;
