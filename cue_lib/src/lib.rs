// #![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod internal;

pub mod core;
pub mod discid;
pub mod error;
pub mod probe;

#[cfg(feature = "metadata")]
pub mod metadata;

#[cfg(feature = "serde")]
pub mod serde;
