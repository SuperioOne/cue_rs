use cue_lib::{
  core::cue_str::CueStr, metadata::VorbisTagName, probe::vorbis_remark::VorbisRemarkIter,
};
use std::collections::HashMap;

pub type MetadataMap<'a> = HashMap<VorbisTagName, Vec<CueStr<'a>>>;

pub fn metadata_from_remarks<'a>(vorbis_remarks: VorbisRemarkIter<'a>) -> Option<MetadataMap<'a>> {
  let mut metadata_map = MetadataMap::new();

  for metadata in vorbis_remarks {
    let list = metadata_map.entry(metadata.tag).or_default();
    list.push(metadata.value);
  }

  if metadata_map.is_empty() {
    None
  } else {
    Some(metadata_map)
  }
}
