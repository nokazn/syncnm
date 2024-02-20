use crate::core::Result;

pub fn both_and_then<A, B>(result_a: Result<A>, result_b: Result<B>) -> Result<(A, B)> {
  result_a.and_then(|a| result_b.map(|b| (a, b)))
}
