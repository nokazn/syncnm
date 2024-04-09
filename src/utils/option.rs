pub fn both_and_then<A, B>(option_a: Option<A>, option_b: Option<B>) -> Option<(A, B)> {
  option_a.and_then(|a| option_b.map(|b| (a, b)))
}
