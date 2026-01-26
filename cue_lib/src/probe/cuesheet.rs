use super::{
  builder::CueProbeBuilder,
  remark::RemarkIter,
  track::{TrackListProbe, Tracks},
};
use crate::{
  core::{album_file::AlbumFile, command::Command, cue_str::CueStr},
  error::{CueLibError, ParseError, ParseErrorKind},
  internal::{lexer::CueLexer, tokenizer::CueTokenizer},
};

pub struct CueSheetProbe<'a> {
  /// Catalog number for the release (CATALOG command)
  pub(super) catalog: Option<CueStr<'a>>,

  /// CD-TEXT file name (CDTEXTFILE command)
  pub(super) cdtextfile: Option<CueStr<'a>>,

  /// Main audio file associated with the album (FILE command)
  pub(super) file: Option<AlbumFile<'a>>,

  /// Performer name for the entire album (PERFORMER command)
  pub(super) performer: Option<CueStr<'a>>,

  /// Songwriter name for the entire album (SONGWRITER command)
  pub(super) songwriter: Option<CueStr<'a>>,

  /// Album title (TITLE command)
  pub(super) title: Option<CueStr<'a>>,

  /// Collection of track-level probe data
  pub(super) tracks_probe: TrackListProbe<'a>,

  /// Slice containing the complete album portion of the cuesheet
  pub(super) album_buffer: &'a str,
}

impl<'a> CueSheetProbe<'a> {
  pub fn new(cuesheet: &'a str) -> Result<Self, CueLibError> {
    let tokenizer = CueTokenizer::new(cuesheet);
    let mut lexer = CueLexer::new(tokenizer);
    let mut builder = CueProbeBuilder::new();
    let mut album_buffer_end = 0;

    'PARSER: while let Some(command) = lexer.next_command()? {
      match command {
        Command::Catalog { value } => builder.set_catalog(value),
        Command::CdTextFile { value } => builder.set_cdtextfile(value),
        Command::File { value } => builder.set_file(value),
        Command::Title { value } => builder.set_title(value),
        Command::Performer { value } => builder.set_performer(value),
        Command::SongWriter { value } => builder.set_songwriter(value),
        Command::Remark { .. } => Ok(()),
        Command::Track { value } => {
          // exit condition
          builder
            .set_tracks_probe(TrackListProbe::new(lexer.snapshot(), value))
            .map_err(|kind| ParseError::new_with_position(kind, lexer.position()))?;

          break 'PARSER;
        }
        _ => Err(ParseErrorKind::InvalidCommandUsage),
      }
      .map_err(|kind| ParseError::new_with_line(kind, lexer.position().line))?;

      album_buffer_end = lexer.cursor_position();
    }

    let probe = builder
      .build(&cuesheet[0..album_buffer_end])
      .map_err(|kind| ParseError::new_with_position(kind, lexer.position()))?;

    Ok(probe)
  }

  /// Iterates over probe to verify if cuesheet is valid.
  pub fn verify(cuesheet: &str) -> Result<(), CueLibError> {
    let probe = CueSheetProbe::new(cuesheet)?;
    let mut tracks = probe.tracks();

    'EXHAUST_TRACKS: loop {
      match tracks.next_track()? {
        Some(track) => {
          let mut indexes = track.indexes();

          'EXHAUST_INDEXES: loop {
            if let None = indexes.next_index()? {
              break 'EXHAUST_INDEXES;
            }
          }
        }
        None => break 'EXHAUST_TRACKS,
      }
    }

    Ok(())
  }

  /// Returns a reference to the album title if present.
  #[inline]
  pub const fn album_title(&self) -> Option<&CueStr<'a>> {
    self.title.as_ref()
  }

  /// Returns a reference to the performer name if present.
  #[inline]
  pub const fn performer(&self) -> Option<&CueStr<'a>> {
    self.performer.as_ref()
  }

  /// Returns a reference to the catalog number if present.
  #[inline]
  pub const fn catalog(&self) -> Option<&CueStr<'a>> {
    self.catalog.as_ref()
  }

  /// Returns a reference to the songwriter name if present.
  #[inline]
  pub const fn songwriter(&self) -> Option<&CueStr<'a>> {
    self.songwriter.as_ref()
  }

  /// Returns a reference to the CD-TEXT file name if present.
  #[inline]
  pub const fn cdtextfile(&self) -> Option<&CueStr<'a>> {
    self.cdtextfile.as_ref()
  }

  /// Returns a reference to the main audio file information if present.
  #[inline]
  pub const fn file_info(&self) -> Option<&AlbumFile<'a>> {
    self.file.as_ref()
  }

  /// Returns an iterator over the tracks in the cuesheet.
  #[inline]
  pub const fn tracks(&self) -> Tracks<'a> {
    self.tracks_probe.iter()
  }

  /// Returns an iterator over the remarks in the album portion of the cuesheet.
  #[inline]
  pub const fn remarks(&self) -> RemarkIter<'a> {
    RemarkIter::new(self.album_buffer)
  }
}
