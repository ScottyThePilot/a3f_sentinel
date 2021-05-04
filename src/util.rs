pub trait ResultExt<T, E> {
  fn report_with(self, msg: &str)
  where E: std::fmt::Debug;

  fn report(self)
  where E: std::fmt::Debug;

  fn ignore(self);
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
  #[inline]
  fn report_with(self, msg: &str)
  where E: std::fmt::Debug {
    if let Err(e) = self {
      println!("{}: {:?}", msg, e);
    };
  }

  #[inline]
  fn report(self)
  where E: std::fmt::Debug {
    if let Err(e) = self {
      println!("{:?}", e);
    };
  }

  #[inline]
  fn ignore(self) {}
}
