use super::remark::RemarkIter;
use crate::{
  core::{
    command::Command,
    cue_str::CueStr,
    flags::TrackFlag,
    timestamp::CueTimeStamp,
    track::{DataType, IndexNo, Track, TrackIndex, TrackNo},
  },
  discid::isrc::Isrc,
  error::{CueLibError, ParseError, ParseErrorKind},
  internal::lexer::CueLexer,
  probe::builder::TrackProbeBuilder,
};

#[derive(Clone)]
pub struct TrackListProbe<'a> {
  lexer: CueLexer<'a>,
  initial_track: Track,
}

#[derive(Clone)]
pub struct TrackIndexProbe<'a> {
  lexer: CueLexer<'a>,
}

#[derive(Clone)]
pub struct TrackProbe<'a> {
  /// Track number and basic track information
  pub(super) track: Track,

  /// Track-specific flags (such as preemphasis, copy permission, etc.)
  pub(super) flags: Option<TrackFlag>,

  /// International Standard Recording Code for the track
  pub(super) isrc: Option<Isrc>,

  /// Post-gap time stamp (POSTGAP command)
  pub(super) postgap: Option<CueTimeStamp>,

  /// Pre-gap time stamp (PREGAP command)
  pub(super) pregap: Option<CueTimeStamp>,

  /// Performer name for this specific track (PERFORMER command)
  pub(super) performer: Option<CueStr<'a>>,

  /// Songwriter name for this specific track (SONGWRITER command)
  pub(super) songwriter: Option<CueStr<'a>>,

  /// Track title (TITLE command)
  pub(super) title: Option<CueStr<'a>>,

  /// Index-level probe data for this track
  pub(super) index_probe: TrackIndexProbe<'a>,

  /// Slice containing the complete track portion of the cuesheet
  pub(super) track_buffer: &'a str,
}

pub struct Tracks<'a> {
  lexer: CueLexer<'a>,
  track: Option<Track>,
}

pub struct TrackIndexes<'a> {
  lexer: CueLexer<'a>,
  previous_index: Option<TrackIndex>,
}

impl<'a> TrackProbe<'a> {
  #[inline]
  pub const fn track_info(&self) -> &Track {
    &self.track
  }

  #[inline]
  pub const fn track_data_type(&self) -> DataType {
    self.track.data_type
  }

  #[inline]
  pub const fn track_no(&self) -> TrackNo {
    self.track.track_no
  }

  #[inline]
  pub const fn isrc(&self) -> Option<&Isrc> {
    self.isrc.as_ref()
  }

  #[inline]
  pub const fn flags(&self) -> Option<&TrackFlag> {
    self.flags.as_ref()
  }

  #[inline]
  pub const fn postgap(&self) -> Option<&CueTimeStamp> {
    self.postgap.as_ref()
  }

  #[inline]
  pub const fn pregap(&self) -> Option<&CueTimeStamp> {
    self.pregap.as_ref()
  }

  #[inline]
  pub const fn performer(&self) -> Option<&CueStr<'a>> {
    self.performer.as_ref()
  }

  #[inline]
  pub const fn songwriter(&self) -> Option<&CueStr<'a>> {
    self.songwriter.as_ref()
  }

  #[inline]
  pub const fn title(&self) -> Option<&CueStr<'a>> {
    self.title.as_ref()
  }

  #[inline]
  pub const fn indexes(&self) -> TrackIndexes<'a> {
    self.index_probe.iter()
  }

  #[inline]
  pub fn remarks(&self) -> RemarkIter<'a> {
    RemarkIter::new(self.track_buffer)
  }
}

impl<'a> TrackListProbe<'a> {
  #[inline]
  pub(super) const fn iter(&self) -> Tracks<'a> {
    Tracks {
      lexer: self.lexer.snapshot(),
      track: Some(self.initial_track),
    }
  }

  #[inline]
  pub(super) const fn new(lexer: CueLexer<'a>, initial_track: Track) -> Self {
    Self {
      lexer,
      initial_track,
    }
  }
}

impl<'a> TrackIndexProbe<'a> {
  #[inline]
  const fn iter(&self) -> TrackIndexes<'a> {
    TrackIndexes {
      lexer: self.lexer.snapshot(),
      previous_index: None,
    }
  }
}

impl<'a> Tracks<'a> {
  pub fn next_track(&mut self) -> Result<Option<TrackProbe<'a>>, CueLibError> {
    if let Some(curr_track) = self.track {
      let index_probe = TrackIndexProbe {
        lexer: self.lexer.snapshot(),
      };
      let mut builder = TrackProbeBuilder::new(index_probe, curr_track);
      let track_buffer_start = self.lexer.position().cursor_index;
      let mut track_buffer_end = track_buffer_start;

      'PARSER: loop {
        match self.lexer.next_command()? {
          Some(Command::Index { .. }) => Ok(()),
          Some(Command::Remark { .. }) => Ok(()),
          Some(Command::Flags { value }) => builder.set_flags(value),
          Some(Command::ISRC { value }) => builder.set_isrc(value),
          Some(Command::Performer { value }) => builder.set_performer(value),
          Some(Command::Postgap { value }) => builder.set_postgap(value),
          Some(Command::Pregap { value }) => builder.set_pregap(value),
          Some(Command::SongWriter { value }) => builder.set_songwriter(value),
          Some(Command::Title { value }) => builder.set_title(value),
          Some(Command::Track { value }) => {
            // Track no's must be sequential
            if curr_track.track_no.saturating_add(1) == value.track_no {
              self.track = Some(value);
              break 'PARSER;
            } else {
              Err(ParseErrorKind::InvalidTrackNo)
            }
          }
          None => {
            self.track = None;
            break 'PARSER;
          }
          Some(_) => Err(ParseErrorKind::InvalidCueSheetFormat),
        }
        .map_err(|kind| ParseError::new_with_position(kind, self.lexer.position()))?;

        track_buffer_end = self.lexer.position().cursor_index;
      }

      let track_buffer = &self.lexer.as_raw_buffer()[track_buffer_start..track_buffer_end];
      let probe = builder
        .build(track_buffer)
        .map_err(|kind| ParseError::new_with_position(kind, self.lexer.position()))?;

      return Ok(Some(probe));
    } else {
      Ok(None)
    }
  }
}

impl<'a> TrackIndexes<'a> {
  pub fn next_index(&mut self) -> Result<Option<TrackIndex>, CueLibError> {
    const START_INDEX: IndexNo = unsafe { IndexNo::new_unchecked(1) };

    loop {
      match self.lexer.next_command()? {
        Some(Command::Index { value }) => {
          let is_valid = match self.previous_index {
            Some(previous) => {
              value.index_no == previous.index_no.saturating_add(1)
                && value.timestamp >= previous.timestamp
            }
            None => value.index_no <= START_INDEX,
          };

          if is_valid {
            self.previous_index = Some(value);
            return Ok(Some(value));
          } else {
            let parse_error = ParseError::new_with_position(
              ParseErrorKind::InvalidTrackIndex,
              self.lexer.position(),
            );

            return Err(parse_error.into());
          }
        }
        Some(Command::Track { .. }) | None => {
          if self.previous_index.is_none() {
            let parse_error = ParseError::new_with_position(
              ParseErrorKind::InvalidCommandFormat,
              self.lexer.position(),
            );

            return Err(parse_error.into());
          } else {
            return Ok(None);
          }
        }
        Some(_) => continue,
      }
    }
  }
}
