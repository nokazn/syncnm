mod tests {
  #[macro_export]
  macro_rules! test_each {
    ($name:ident, $($suffix:expr => $case:expr,)*) => {
      paste::item! {
        $(
          #[test]
          fn [< $name _ $suffix >]() {
            [< $name _each >]($case);
          }
        )*
      }
    };
  }

  #[macro_export]
  macro_rules! test_each_serial {
    ($name:ident, $($suffix:expr => $case:expr,)*) => {
      paste::item! {
        $(
          #[test]
          #[serial_test::serial]
          fn [< $name _ $suffix >]() {
            [< $name _each >]($case);
          }
        )*
      }
    };
  }
}
