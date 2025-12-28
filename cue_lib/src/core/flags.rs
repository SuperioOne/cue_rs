use super::error::FlagParseError;
use crate::internal::bitflag::define_bitflag;

define_bitflag!(
  pub TrackFlag u8,
  values = [
    /// Digital copy permitted
    (DCP, 1),
    /// Four channel audio
    (FOUR_CHANNEL, 1 << 1),
    /// Pre-emphasis enabled on audio track
    (PRE, 1 << 2),
    /// Serial copy management system
    (SCMS, 1 << 3)
  ]
);

impl core::str::FromStr for TrackFlag {
  type Err = FlagParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    const TABLE: [(&'static str, TrackFlag); 4] = [
      ("DCP", TrackFlag::DCP),
      ("4CH", TrackFlag::FOUR_CHANNEL),
      ("SCMS", TrackFlag::SCMS),
      ("PRE", TrackFlag::PRE),
    ];

    for (name, value) in TABLE {
      if name.eq_ignore_ascii_case(s) {
        return Ok(value);
      }
    }

    Err(FlagParseError)
  }
}
