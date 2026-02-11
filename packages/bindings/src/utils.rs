use takeoff_core::error::{TakeoffError, TakeoffResult};

/// Helper function to lock a mutex and convert poison errors to TakeoffError.
pub fn lock_mutex<'a, T>(
  guard: std::result::Result<
    std::sync::MutexGuard<'a, T>,
    std::sync::PoisonError<std::sync::MutexGuard<'a, T>>,
  >,
  resource: &str,
) -> TakeoffResult<std::sync::MutexGuard<'a, T>> {
  guard.map_err(|_| TakeoffError::poison_error(resource))
}
