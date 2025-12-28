use self::cuesheet::CueSheetProbe;
use crate::error::CueLibError;

mod builder;

pub mod cuesheet;
pub mod remark;
pub mod track;

pub fn probe_cuesheet<'a>(cuesheet: &'a str) -> Result<CueSheetProbe<'a>, CueLibError> {
  CueSheetProbe::attach_to(cuesheet)
}
