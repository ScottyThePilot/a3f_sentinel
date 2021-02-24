#[macro_export]
macro_rules! ignore {
  ($expr:expr) => {
    if let Err(err) = $expr {
      println!("Error: {:?}", err);
    };
  };
  ($arg:tt, $expr:expr) => {
    if let Err(err) = $expr {
      println!($arg, err);
    };
  };
}
