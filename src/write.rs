use anyhow::{bail, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const BITS_PER_SAMPLE: u16 = 16;

/// Creates a vector of WAV bytes from audio data.
pub fn create_bytes(audiodata: impl AsRef<[Vec<i16>]>, samplerate: u32) -> Result<Vec<u8>> {
    let audiodata = audiodata.as_ref();

    if audiodata.len() < 1 || audiodata.len() > 2 {
        bail!("unsupported number of channels");
    }

    let num_channels = audiodata.len() as u16;

    let audiodata_size =
        audiodata[0].len() as u32 * num_channels as u32 * BITS_PER_SAMPLE as u32 / 8;

    let mut buf = vec![];
    write_riff_chunk(&mut buf, audiodata_size)?;
    write_fmt_chunk(&mut buf, num_channels, samplerate)?;
    write_data_chunk(&mut buf, audiodata, audiodata_size)?;

    Ok(buf)
}

/// Writes audio data into a WAV file.
pub fn write(
    audiodata: impl AsRef<[Vec<i16>]>,
    samplerate: u32,
    path: impl AsRef<Path>,
) -> Result<()> {
    let bytes = create_bytes(audiodata, samplerate)?;

    let mut writer = BufWriter::new(File::create(path)?);
    writer.write_all(&bytes)?;

    Ok(())
}

fn write_riff_chunk(buf: &mut Vec<u8>, audiodata_size: u32) -> Result<()> {
    write!(buf, "RIFF")?;

    let chunksize = 36 + audiodata_size;
    buf.extend_from_slice(&chunksize.to_le_bytes());

    write!(buf, "WAVE")?;
    Ok(())
}

fn write_fmt_chunk(buf: &mut Vec<u8>, num_channels: u16, samplerate: u32) -> Result<()> {
    const CHUNKSIZE: u32 = 16;
    const AUDIOFORMAT: u16 = 1;

    let byterate = samplerate * num_channels as u32 * BITS_PER_SAMPLE as u32 / 8;
    let block_align = num_channels * BITS_PER_SAMPLE / 8;

    write!(buf, "fmt ")?;
    buf.extend_from_slice(&CHUNKSIZE.to_le_bytes());
    buf.extend_from_slice(&AUDIOFORMAT.to_le_bytes());
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&samplerate.to_le_bytes());
    buf.extend_from_slice(&byterate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());

    Ok(())
}

fn write_data_chunk(buf: &mut Vec<u8>, audiodata: &[Vec<i16>], audiodata_size: u32) -> Result<()> {
    write!(buf, "data")?;
    buf.extend_from_slice(&audiodata_size.to_le_bytes());

    for (&left, &right) in audiodata[0].iter().zip(audiodata[1].iter()) {
        buf.extend_from_slice(&left.to_le_bytes());
        buf.extend_from_slice(&right.to_le_bytes());
    }

    Ok(())
}
