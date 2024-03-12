#[macro_export]
macro_rules! btree_map {

  ($($key:expr => $value:expr,)*) => {
    {
      let mut map = BTreeMap::new();
      $(
        map.insert($key, $value);
      )*
      map
    }
  };
}
