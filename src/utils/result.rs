#[cfg(test)]
use anyhow::Result;

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
