use super::Command;
use cue_lib::probe::CueSheetProbe;

pub struct CommandVerify<'a> {
  cuesheet: &'a str,
}

impl<'a> CommandVerify<'a> {
  #[inline]
  pub const fn new(cuesheet: &'a str) -> Self {
    Self { cuesheet }
  }
}

impl<'a> Command for &'a CommandVerify<'a> {
  type Error = cue_lib::error::CueLibError;

  #[inline]
  fn run(self) -> Result<(), cue_lib::error::CueLibError> {
    CueSheetProbe::verify(self.cuesheet)
  }
}
