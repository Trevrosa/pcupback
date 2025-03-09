use std::ops::Deref;

use rocket::State;
use sqlx::{Database, Pool};

/// A one-fn extension trait, see [`ToExecutor::to_db`].
pub trait ToExecutor<T: Database>: Deref<Target = State<Pool<T>>> {
    /// Convert a rocket-managed db to one which implements [`sqlx::Executor`]
    ///
    ///     &State<Pool<T>> where T: Database
    ///
    ///     deref &State<Pool<T>> => State<Pool<T>>
    ///
    ///     deref State<Pool<T>> => Pool<T>
    ///
    ///     &Pool<T>
    ///
    /// we end up with &[`Pool<T>`], which implements [`sqlx::Executor`]
    #[inline]
    #[must_use]
    fn to_db(&self) -> &Pool<T> {
        // auto-deref'ed
        self
    }
}

/// Blanket impl.
impl<T, D: Database> ToExecutor<D> for T where T: Deref<Target = State<Pool<D>>> {}
