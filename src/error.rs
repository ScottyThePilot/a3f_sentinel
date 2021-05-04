error_enum!{
  pub enum Error {
    Io(std::io::Error),
    Format(singlefile::serde_multi::Error),
    Serenity(serenity::prelude::SerenityError),
    Custom(&'static str)
  }
}

impl From<singlefile::Error> for Error {
  fn from(value: singlefile::Error) -> Error {
    match value {
      singlefile::Error::Io(err) => Error::Io(err.into()),
      singlefile::Error::Format(err) => Error::Format(err)
    }
  }
}
