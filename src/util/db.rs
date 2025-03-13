use std::ops::Deref;

use rocket::State;
use sqlx::{Database, Pool};

/// A extension trait for all [`State<Pool<T>>`], see [`PoolStateExt::to_db`].
pub trait PoolStateExt<T: Database>: Deref<Target = State<Pool<T>>> {
    /// Unwrap a rocket-managed database pool to one which implements [`sqlx::Executor`].
    ///
    /// The trait bound for `Self` enforces that `Self` dereferences to [`State<Pool<T>>`].
    ///
    /// The conversion goes like this:
    /// ```
    ///     &State<Pool<T>> where T: Database
    ///
    ///     deref &State<Pool<T>> => State<Pool<T>>
    ///
    ///     deref State<Pool<T>> => Pool<T>
    ///
    ///     &Pool<T>
    ///```
    /// We end up with `&Pool<T>`, which implements [`sqlx::Executor`].
    ///
    /// # Note
    ///
    /// Because of the function definition and `Self` trait bounds, the compiler auto-derefs for us. So instead of `return &**self`, the actual implementation is just `return self`.
    #[inline]
    #[must_use]
    fn to_db(&self) -> &Pool<T> {
        // auto-deref'ed
        self
    }
}

// Blanket impl
impl<T, D: Database> PoolStateExt<D> for T where T: Deref<Target = State<Pool<D>>> {}
