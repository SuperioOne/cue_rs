#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub(crate) mod internal;

pub mod core;
pub mod discid;
pub mod error;
pub mod probe;
