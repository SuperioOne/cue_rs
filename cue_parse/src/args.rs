use clap::{Parser, Subcommand, ValueEnum};
use std::{ffi::OsString, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
  #[arg(short, long)]
  pub input: Option<PathBuf>,

  #[arg(short, long)]
  pub verbose: Option<VerboseLevel>,

  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  Verify,
  ConvertJson {
    #[arg(short, long)]
    output_file: Option<PathBuf>,

    #[arg(short, long)]
    metadata: Option<MetadataFormat>,

    #[arg(short, long)]
    pretty_print: bool,
  },
  Query {
    input: OsString,
  },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum MetadataFormat {
  Vorbis,
  Id3v2,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum VerboseLevel {
  Default,
  Full,
  Quiet,
}

impl Args {
  #[inline]
  pub fn init() -> Self {
    Self::parse()
  }
}

impl Default for VerboseLevel {
  #[inline]
  fn default() -> Self {
    Self::Default
  }
}
