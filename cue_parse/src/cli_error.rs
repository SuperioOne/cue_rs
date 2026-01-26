use crate::args::VerboseLevel;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct CliError<E> {
  verbose_level: VerboseLevel,
  error: E,
}

impl<E> CliError<E>
where
  E: ErrorFormat,
{
  #[inline]
  pub const fn new(error: E) -> Self {
    Self {
      error,
      verbose_level: VerboseLevel::Default,
    }
  }

  #[inline]
  pub const fn new_with_verbose(error: E, verbose_level: VerboseLevel) -> Self {
    Self {
      error,
      verbose_level,
    }
  }
}

impl<E> core::fmt::Display for CliError<E>
where
  E: ErrorFormat,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("cue_parse: ")?;
    self.error.fmt(f, self.verbose_level)
  }
}

pub trait ErrorFormat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>, verbose_level: VerboseLevel) -> std::fmt::Result;
}

impl ErrorFormat for std::io::Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>, verbose_level: VerboseLevel) -> std::fmt::Result {
    match verbose_level {
      VerboseLevel::Quiet => Ok(()),
      _ => std::fmt::Display::fmt(&self, f),
    }
  }
}

impl ErrorFormat for cue_lib::error::CueLibError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _: VerboseLevel) -> std::fmt::Result {
    std::fmt::Display::fmt(&self, f)
  }
}

impl<E> core::error::Error for CliError<E> where E: ErrorFormat + Display + Debug {}

macro_rules! cli_stderr {
  ($error:expr) => {{
    let error = $crate::cli_error::CliError::new($error);
    eprintln!("{}", $error)
  }};

  ($error:expr, verbosity = $verbosity:expr) => {{
    let error = $crate::cli_error::CliError::new_with_verbose($error, $verbosity);
    eprintln!("{}", error);
  }};
}

pub(crate) use cli_stderr;
