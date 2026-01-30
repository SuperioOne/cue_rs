use super::remark::RemarkIter;
use crate::metadata::VorbisComment;

pub struct VorbisRemarkIter<'a> {
  inner: RemarkIter<'a>,
}

impl<'a> Iterator for VorbisRemarkIter<'a> {
  type Item = VorbisComment<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(remark) = self.inner.next() {
      match VorbisComment::try_from_line(remark) {
        Ok(vorbis_comment) => return Some(vorbis_comment),
        Err(_) => continue,
      }
    }

    None
  }
}

impl<'a> From<RemarkIter<'a>> for VorbisRemarkIter<'a> {
  #[inline]
  fn from(value: RemarkIter<'a>) -> Self {
    Self { inner: value }
  }
}
