use self::{
  error::ConvertError,
  metadata::{MetadataMap, metadata_from_remarks},
};

use super::Command;
use cue_lib::{
  core::{
    album_file::AlbumFile,
    cue_str::CueStr,
    flags::TrackFlag,
    timestamp::CueTimeStamp,
    track::{DataType, TrackNo},
  },
  discid::isrc::Isrc,
  probe::{
    CueSheetProbe,
    track::{TrackSubIndexes, Tracks},
  },
};
use serde::Serialize;
use std::{
  fs::OpenOptions,
  io::{BufWriter, stdout},
  path::PathBuf,
};

mod error;
mod metadata;

pub struct ConvertCommand<'a> {
  cuesheet: &'a str,
  allow_metadata_remarks: bool,
  output_file: Option<PathBuf>,
  pretty_print: bool,
}

#[derive(Serialize)]
struct CueSheet<'a> {
  pub catalog: Option<CueStr<'a>>,
  pub cdtextfile: Option<CueStr<'a>>,
  pub file: Option<AlbumFile<'a>>,
  pub performer: Option<CueStr<'a>>,
  pub remark_metadata: Option<MetadataMap<'a>>,
  pub songwriter: Option<CueStr<'a>>,
  pub title: Option<CueStr<'a>>,
  pub tracks: Vec<TrackInfo<'a>>,
}

#[derive(Serialize)]
struct TrackInfo<'a> {
  pub data_type: DataType,
  pub flags: Option<TrackFlag>,
  pub isrc: Option<Isrc>,
  pub performer: Option<CueStr<'a>>,
  pub postgap: Option<CueTimeStamp>,
  pub pregap: Option<CueTimeStamp>,
  pub remark_metadata: Option<MetadataMap<'a>>,
  pub songwriter: Option<CueStr<'a>>,
  pub sub_indexes: Option<Vec<CueTimeStamp>>,
  pub time_info: TimeInfo,
  pub title: Option<CueStr<'a>>,
  pub track_no: TrackNo,
}

#[derive(Serialize, Default)]
struct TimeInfo {
  start: u128,
  end: Option<u128>,
  pregap_start: Option<u128>,
  duration: Option<u128>,
}

impl<'a> ConvertCommand<'a> {
  #[inline]
  pub const fn new(cuesheet: &'a str) -> Self {
    Self {
      cuesheet,
      pretty_print: false,
      allow_metadata_remarks: false,
      output_file: None,
    }
  }

  #[inline]
  pub const fn set_pretty_print(mut self, value: bool) -> Self {
    self.pretty_print = value;
    self
  }

  #[inline]
  pub const fn set_metadata_remarks(mut self, value: bool) -> Self {
    self.allow_metadata_remarks = value;
    self
  }

  #[inline]
  pub fn set_output_file(mut self, value: Option<PathBuf>) -> Self {
    self.output_file = value;
    self
  }

  #[inline]
  fn process_tracks(
    &self,
    mut cuesheet: CueSheet<'a>,
    mut track_probe: Tracks<'a>,
  ) -> Result<CueSheet<'a>, ConvertError> {
    while let Some(track) = track_probe.next_track()? {
      let mut track_info = TrackInfo {
        data_type: track.track_data_type(),
        flags: track.flags(),
        isrc: track.isrc(),
        performer: track.performer(),
        postgap: track.postgap(),
        pregap: track.pregap(),
        remark_metadata: None,
        songwriter: track.songwriter(),
        sub_indexes: None,
        time_info: TimeInfo {
          start: track.start_index().as_millis(),
          pregap_start: track.pregap_index().map(|v| v.as_millis()),
          end: None,
          duration: None,
        },
        title: track.title(),
        track_no: track.track_no(),
      };

      Self::process_sub_indexes(&mut track_info, track.sub_indexes())?;

      if self.allow_metadata_remarks {
        track_info.remark_metadata = metadata_from_remarks(track.vorbis_comments());
      }

      cuesheet.tracks.push(track_info);
    }

    Self::calc_track_times(&mut cuesheet.tracks);

    Ok(cuesheet)
  }

  fn process_sub_indexes(
    track: &mut TrackInfo<'a>,
    mut indexes: TrackSubIndexes<'a>,
  ) -> Result<(), ConvertError> {
    let mut sub_indexes = Vec::new();
    while let Some(index) = indexes.next_index()? {
      sub_indexes.push(index.timestamp);
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
    let probe = CueSheetProbe::new(self.cuesheet)?;
    let mut cuesheet = CueSheet {
      catalog: probe.catalog(),
      cdtextfile: probe.cdtextfile(),
      file: probe.file_info(),
      performer: probe.performer(),
      remark_metadata: None,
      songwriter: probe.songwriter(),
      title: probe.album_title(),
      tracks: Vec::new(),
    };

    if self.allow_metadata_remarks {
      cuesheet.remark_metadata = metadata_from_remarks(probe.vorbis_comments());
    }

    let cuesheet = self.process_tracks(cuesheet, probe.tracks())?;

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
