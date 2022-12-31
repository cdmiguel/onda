//! Simple PCM-16-bit-integer only WAV file reader and writer.
//! Spec source: http://tiny.systems/software/soundProgrammer/WavFormatDocs.pdf

mod read;
mod write;

pub use read::*;
pub use write::*;
