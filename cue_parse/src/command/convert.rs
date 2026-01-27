use super::Command;
use crate::{
  args::{MetadataFormat, VerboseLevel},
  cli_error::ErrorFormat,
};
use cue_lib::{
  error::CueLibError,
  probe::{CueSheetProbe, track::TrackIndexes},
};
use serde::Serialize;
use std::{
  borrow::Cow,
  fs::OpenOptions,
  io::{BufWriter, stdout},
  path::PathBuf,
};

pub struct ConvertCommand<'a> {
  cuesheet: &'a str,
  allow_metadata_remarks: Option<MetadataFormat>,
  output_file: Option<PathBuf>,
  pretty_print: bool,
}

#[derive(Serialize)]
pub struct CueSheet<'a> {
  pub catalog: Option<Cow<'a, str>>,
  pub cdtextfile: Option<Cow<'a, str>>,
  pub file: Option<Cow<'a, str>>,
  pub file_type: Option<&'a str>,
  pub performer: Option<Cow<'a, str>>,
  pub songwriter: Option<Cow<'a, str>>,
  pub title: Option<Cow<'a, str>>,
  pub tracks: Vec<TrackInfo<'a>>,
}

#[derive(Serialize)]
pub struct TrackInfo<'a> {
  pub data_type: &'a str,
  pub flags: Option<Vec<&'static str>>,
  pub isrc: Option<String>,
  pub performer: Option<Cow<'a, str>>,
  pub postgap: Option<u128>,
  pub pregap: Option<u128>,
  pub songwriter: Option<Cow<'a, str>>,
  pub sub_indexes: Option<Vec<u128>>,
  pub time_info: TimeInfo,
  pub title: Option<Cow<'a, str>>,
  pub track_no: u8,
}

#[derive(Serialize, Default)]
pub struct TimeInfo {
  start: u128,
  end: Option<u128>,
  pregap_start: Option<u128>,
  duration: Option<u128>,
}

pub enum ConvertError {
  CueLibError(CueLibError),
  IOError(std::io::Error),
  JsonSerializeError(serde_json::error::Error),
  IndexError,
}

impl<'a> ConvertCommand<'a> {
  #[inline]
  pub const fn new(cuesheet: &'a str) -> Self {
    Self {
      cuesheet,
      pretty_print: false,
      allow_metadata_remarks: None,
      output_file: None,
    }
  }

  #[inline]
  pub const fn set_pretty_print(mut self, value: bool) -> Self {
    self.pretty_print = value;
    self
  }

  #[inline]
  pub const fn set_metadata_remarks(mut self, value: Option<MetadataFormat>) -> Self {
    self.allow_metadata_remarks = value;
    self
  }

  #[inline]
  pub fn set_output_file(mut self, value: Option<PathBuf>) -> Self {
    self.output_file = value;
    self
  }

  #[inline]
  fn process_cuesheet(&self) -> Result<CueSheet<'a>, ConvertError> {
    let probe = CueSheetProbe::new(self.cuesheet)?;
    let mut cuesheet = CueSheet {
      catalog: probe.catalog().map(|v| v.into()),
      cdtextfile: probe.cdtextfile().map(|v| v.into()),
      file: probe.file_info().map(|v| v.name.into()),
      file_type: probe.file_info().map(|v| v.file_type.as_str()),
      performer: probe.performer().map(|v| v.into()),
      songwriter: probe.songwriter().map(|v| v.into()),
      title: probe.album_title().map(|v| v.into()),
      tracks: Vec::new(),
    };

    Self::process_tracks(&probe, &mut cuesheet)?;

    Ok(cuesheet)
  }

  #[inline]
  fn process_tracks(
    probe: &CueSheetProbe<'a>,
    cuesheet: &mut CueSheet<'a>,
  ) -> Result<(), ConvertError> {
    let mut track_probe = probe.tracks();

    while let Some(track) = track_probe.next_track()? {
      let mut track_info = TrackInfo {
        data_type: track.track_data_type().as_str(),
        flags: None,
        isrc: track.isrc().map(|v| v.to_string()),
        performer: track.performer().map(|v| v.into()),
        postgap: track.postgap().map(|v| v.as_millis()),
        pregap: track.pregap().map(|v| v.as_millis()),
        songwriter: track.songwriter().map(|v| v.into()),
        sub_indexes: None,
        time_info: TimeInfo::default(),
        title: track.title().map(|v| v.into()),
        track_no: track.track_no().into_inner(),
      };

      Self::process_indexes(&mut track_info, track.indexes())?;
      cuesheet.tracks.push(track_info);
    }

    Self::calc_track_times(&mut cuesheet.tracks);

    Ok(())
  }

  fn process_indexes(
    track: &mut TrackInfo<'a>,
    mut indexes: TrackIndexes<'a>,
  ) -> Result<(), ConvertError> {
    match indexes.next_index()? {
      Some(index) => match index.index_no.as_ref() {
        0 => {
          track.time_info.pregap_start = Some(index.timestamp.as_millis());

          if let Some(start) = indexes.next_index()?
            && *start.index_no.as_ref() == 1u8
          {
            track.time_info.start = start.timestamp.as_millis();
          } else {
            return Err(ConvertError::IndexError);
          }
        }
        1 => {
          track.time_info.start = index.timestamp.as_millis();
        }
        _ => return Err(ConvertError::IndexError),
      },
      None => {
        return Err(ConvertError::IndexError);
      }
    };

    let mut sub_indexes = Vec::new();
    while let Some(index) = indexes.next_index()? {
      sub_indexes.push(index.timestamp.as_millis());
    }

    if !sub_indexes.is_empty() {
      track.sub_indexes = Some(sub_indexes);
    }

    Ok(())
  }

  #[inline]
  fn calc_track_times(tracks: &mut Vec<TrackInfo>) {
    let mut track_iter = tracks.iter_mut().peekable();

    while let Some(track) = track_iter.next() {
      if let Some(next_track) = track_iter.peek() {
        let end = if let Some(pregap) = next_track.time_info.pregap_start {
          pregap
        } else {
          next_track.time_info.start
        };

        track.time_info.end = Some(end);
        track.time_info.duration = Some(end - track.time_info.start);
      }
    }
  }
}

impl<'a> Command for &'a ConvertCommand<'a> {
  type Error = ConvertError;

  fn run(self) -> Result<(), ConvertError> {
    let cuesheet = self.process_cuesheet()?;
    let target_stream: Box<dyn std::io::Write> = match self.output_file.as_ref() {
      Some(path) => {
        let fd = OpenOptions::new()
          .create(true)
          .write(true)
          .truncate(true)
          .open(path)?;

        Box::new(fd)
      }
      None => Box::new(stdout().lock()),
    };

    let mut buf_writer = BufWriter::new(target_stream);

    if self.pretty_print {
      serde_json::to_writer_pretty(&mut buf_writer, &cuesheet)?
    } else {
      serde_json::to_writer(&mut buf_writer, &cuesheet)?
    }

    Ok(())
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

impl ErrorFormat for ConvertError {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
    _: &str,
    verbose_level: crate::args::VerboseLevel,
  ) -> std::fmt::Result {
    if verbose_level == VerboseLevel::Quiet {
      Ok(())
    } else {
      match self {
        ConvertError::IndexError => {
          f.write_str("unhandled logic error, expected an index command but received nothing")
        }
        ConvertError::CueLibError(error) => std::fmt::Display::fmt(&error, f),
        ConvertError::IOError(error) => std::fmt::Display::fmt(&error, f),
        ConvertError::JsonSerializeError(error) => std::fmt::Display::fmt(&error, f),
      }
    }
  }
}
