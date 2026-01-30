use crate::{args::VerboseLevel, cli_error::ErrorFormat};
use cue_lib::error::CueLibError;

pub enum ConvertError {
  CueLibError(CueLibError),
  IOError(std::io::Error),
  JsonSerializeError(serde_json::error::Error),
}

impl ErrorFormat for ConvertError {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
    input_buffer: &str,
    verbose_level: crate::args::VerboseLevel,
  ) -> std::fmt::Result {
    if verbose_level == VerboseLevel::Quiet {
      Ok(())
    } else {
      match self {
        ConvertError::CueLibError(error) => ErrorFormat::fmt(error, f, input_buffer, verbose_level),
        ConvertError::IOError(error) => std::fmt::Display::fmt(&error, f),
        ConvertError::JsonSerializeError(error) => std::fmt::Display::fmt(&error, f),
      }
    }
  }
}

impl From<CueLibError> for ConvertError {
  #[inline]
  fn from(value: CueLibError) -> Self {
    Self::CueLibError(value)
  }
}

impl From<serde_json::error::Error> for ConvertError {
  #[inline]
  fn from(value: serde_json::error::Error) -> Self {
    Self::JsonSerializeError(value)
  }
}

impl From<std::io::Error> for ConvertError {
  #[inline]
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}
