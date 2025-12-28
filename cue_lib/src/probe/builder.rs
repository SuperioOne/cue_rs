use super::{
  cuesheet::CueSheetProbe,
  track::{TrackIndexProbe, TrackListProbe, TrackProbe},
};
use crate::{
  core::{
    album_file::AlbumFile, cue_str::CueStr, flags::TrackFlag, timestamp::CueTimeStamp, track::Track,
  },
  discid::isrc::Isrc,
  error::ParseErrorKind,
};

pub struct CueProbeBuilder<'a> {
  catalog: Option<CueStr<'a>>,
  cdtextfile: Option<CueStr<'a>>,
  file: Option<AlbumFile<'a>>,
  performer: Option<CueStr<'a>>,
  songwriter: Option<CueStr<'a>>,
  title: Option<CueStr<'a>>,
  tracks_probe: Option<TrackListProbe<'a>>,
}

pub struct TrackProbeBuilder<'a> {
  track: Track,
  flags: Option<TrackFlag>,
  isrc: Option<Isrc>,
  postgap: Option<CueTimeStamp>,
  pregap: Option<CueTimeStamp>,
  performer: Option<CueStr<'a>>,
  songwriter: Option<CueStr<'a>>,
  title: Option<CueStr<'a>>,
  index_probe: TrackIndexProbe<'a>,
}

impl<'a> CueProbeBuilder<'a> {
  #[inline]
  pub fn new() -> Self {
    Self {
      catalog: None,
      cdtextfile: None,
      file: None,
      performer: None,
      songwriter: None,
      title: None,
      tracks_probe: None,
    }
  }

  pub const fn set_catalog(&mut self, catalog: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.catalog.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.catalog = Some(catalog);
    Ok(())
  }

  #[inline]
  pub const fn set_cdtextfile(&mut self, cdtextfile: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.cdtextfile.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.cdtextfile = Some(cdtextfile);
    Ok(())
  }

  #[inline]
  pub const fn set_file(&mut self, file: AlbumFile<'a>) -> Result<(), ParseErrorKind> {
    if self.file.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.file = Some(file);
    Ok(())
  }

  #[inline]
  pub const fn set_performer(&mut self, performer: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.performer.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.performer = Some(performer);
    Ok(())
  }

  #[inline]
  pub const fn set_songwriter(&mut self, songwriter: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.songwriter.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.songwriter = Some(songwriter);
    Ok(())
  }

  #[inline]
  pub const fn set_title(&mut self, title: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.title.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.title = Some(title);
    Ok(())
  }

  #[inline]
  pub const fn set_tracks_probe(
    &mut self,
    probe: TrackListProbe<'a>,
  ) -> Result<(), ParseErrorKind> {
    if self.tracks_probe.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.tracks_probe = Some(probe);
    Ok(())
  }

  pub fn build(self, album_buffer: &'a str) -> Result<CueSheetProbe<'a>, ParseErrorKind> {
    let tracks_probe = self
      .tracks_probe
      .ok_or(ParseErrorKind::MissingTrackCommand)?;

    let probe = CueSheetProbe {
      catalog: self.catalog,
      cdtextfile: self.cdtextfile,
      file: self.file,
      performer: self.performer,
      songwriter: self.songwriter,
      title: self.title,
      tracks_probe,
      album_buffer,
    };

    Ok(probe)
  }
}

impl<'a> TrackProbeBuilder<'a> {
  #[inline]
  pub fn new(index_probe: TrackIndexProbe<'a>, track: Track) -> Self {
    Self {
      track,
      index_probe,
      flags: None,
      isrc: None,
      postgap: None,
      pregap: None,
      performer: None,
      songwriter: None,
      title: None,
    }
  }

  #[inline]
  pub const fn set_flags(&mut self, flags: TrackFlag) -> Result<(), ParseErrorKind> {
    if self.flags.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.flags = Some(flags);
    Ok(())
  }

  #[inline]
  pub const fn set_isrc(&mut self, isrc: Isrc) -> Result<(), ParseErrorKind> {
    if self.isrc.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.isrc = Some(isrc);
    Ok(())
  }

  #[inline]
  pub const fn set_postgap(&mut self, postgap: CueTimeStamp) -> Result<(), ParseErrorKind> {
    if self.postgap.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.postgap = Some(postgap);
    Ok(())
  }

  #[inline]
  pub const fn set_pregap(&mut self, pregap: CueTimeStamp) -> Result<(), ParseErrorKind> {
    if self.pregap.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.pregap = Some(pregap);
    Ok(())
  }

  #[inline]
  pub const fn set_performer(&mut self, performer: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.performer.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.performer = Some(performer);
    Ok(())
  }

  #[inline]
  pub const fn set_songwriter(&mut self, songwriter: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.songwriter.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.songwriter = Some(songwriter);
    Ok(())
  }

  #[inline]
  pub const fn set_title(&mut self, title: CueStr<'a>) -> Result<(), ParseErrorKind> {
    if self.title.is_some() {
      return Err(ParseErrorKind::MultipleCommand);
    }

    self.title = Some(title);
    Ok(())
  }

  pub fn build(self, track_buffer: &'a str) -> Result<TrackProbe<'a>, ParseErrorKind> {
    let probe = TrackProbe {
      track: self.track,
      flags: self.flags,
      isrc: self.isrc,
      postgap: self.postgap,
      pregap: self.pregap,
      performer: self.performer,
      songwriter: self.songwriter,
      title: self.title,
      index_probe: self.index_probe,
      track_buffer,
    };

    Ok(probe)
  }
}
