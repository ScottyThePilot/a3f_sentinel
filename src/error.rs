use std::fmt;


#[derive(Debug)]
pub enum Error {
  Ron(ron::error::Error),
  Io(async_std::io::Error),
  Serenity(serenity::prelude::SerenityError),
  Custom(&'static str),
}

impl From<ron::error::Error> for Error {
  fn from(value: ron::error::Error) -> Error {
    Error::Ron(value)
  }
}

impl From<async_std::io::Error> for Error {
  fn from(value: async_std::io::Error) -> Error {
    Error::Io(value)
  }
}

impl From<serenity::prelude::SerenityError> for Error {
  fn from(value: serenity::prelude::SerenityError) -> Error {
    Error::Serenity(value)
  }
}

impl From<&'static str> for Error {
  fn from(value: &'static str) -> Error {
    Error::Custom(value)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Ron(err) => write!(f, "ron {}", err),
      Error::Io(err) => write!(f, "io {}", err),
      Error::Serenity(err) => write!(f, "serenity {}", err),
      Error::Custom(err) => write!(f, "custom {}", err),
    }
  }
}

impl std::error::Error for Error {}
