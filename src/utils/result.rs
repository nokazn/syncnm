use crate::core::Result;

pub fn both_and_then<A, B>(result_a: Result<A>, result_b: Result<B>) -> Result<(A, B)> {
  result_a.and_then(|a| result_b.map(|b| (a, b)))
}

#[cfg(test)]
use std::panic;

#[cfg(test)]
use crate::errors::to_error;

#[cfg(test)]
pub fn convert_panic_to_result<F, T>(f: F) -> Result<T>
where
  F: FnOnce() -> T + panic::UnwindSafe,
{
  panic::catch_unwind(f).map_err(to_error)
}
