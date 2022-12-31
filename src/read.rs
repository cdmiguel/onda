use anyhow::{bail, Error, Result};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// WAV info and audio data. `audiodata` is a vector of channels, and each channel is
/// a vector of 16-bit samples.
#[derive(Clone)]
pub struct WavData {
    pub num_channels: u16,
    pub samplerate: u32,
    pub audiodata: Vec<Vec<i16>>,
}

/// Details about the WAV file.
#[derive(Clone, Copy)]
struct Spec {
    num_channels: u16,
    samplerate: u32,
}

/// Parses a WAV file from a byte slice buffer;
pub fn parse_bytes(buf: impl AsRef<[u8]>) -> Result<WavData> {
    let buf = buf.as_ref();
    let mut offset = 0;

    parse_riff_chunk(buf, &mut offset)?;
    let spec = parse_fmt_chunk(buf, &mut offset)?;
    find_data_offset(buf, &mut offset)?;
    let data = parse_data_chunk(buf, &mut offset, spec)?;

    Ok(WavData {
        num_channels: spec.num_channels,
        samplerate: spec.samplerate,
        audiodata: data,
    })
}

/// Reads a WAV file from the provided path.
pub fn read(path: impl AsRef<Path>) -> Result<WavData> {
    let mut reader = BufReader::new(File::open(path)?);

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    parse_bytes(&buf)
}

fn parse_riff_chunk(buf: &[u8], offset: &mut usize) -> Result<()> {
    if !compare_str_bytes(buf, offset, "RIFF") {
        bail!("not a RIFF file");
    }

    // ignore chunk size
    *offset += 4;

    if !compare_str_bytes(buf, offset, "WAVE") {
        bail!("not a WAVE file");
    }

    Ok(())
}

fn parse_fmt_chunk(buf: &[u8], offset: &mut usize) -> Result<Spec> {
    if parse_str(&buf, offset, 4) != "fmt " {
        bail!("fmt chunk not found");
    }

    if parse_u32(&buf, offset) != 16 {
        bail!("fmt chunk wrong size");
    }

    if parse_u16(buf, offset) != 1 {
        bail!("not a PCM file");
    }

    let num_channels = parse_u16(buf, offset);
    let samplerate = parse_u32(buf, offset);
    let byterate = parse_u32(buf, offset);
    let block_align = parse_u16(buf, offset);
    let bits_per_sample = parse_u16(buf, offset);

    if byterate != samplerate * num_channels as u32 * bits_per_sample as u32 / 8 {
        bail!("byte rate does not match with other parameters");
    }

    if block_align != num_channels * bits_per_sample / 8 {
        bail!("block align does not match with other parameters");
    }

    Ok(Spec {
        num_channels,
        samplerate,
    })
}

fn parse_data_chunk(buf: &[u8], offset: &mut usize, spec: Spec) -> Result<Vec<Vec<i16>>> {
    let size = parse_u32(&buf, offset) as usize;

    if spec.num_channels == 1 {
        let mut samples = vec![];

        while *offset < size {
            samples.push(parse_i16(buf, offset));
        }

        Ok(vec![samples])
    } else if spec.num_channels == 2 {
        let mut samples_l = vec![];
        let mut samples_r = vec![];

        while *offset < size {
            samples_l.push(parse_i16(buf, offset));
            samples_r.push(parse_i16(buf, offset));
        }

        Ok(vec![samples_l, samples_r])
    } else {
        Err(Error::msg("unsupported number of channels"))
    }
}

fn find_data_offset(buf: &[u8], offset: &mut usize) -> Result<()> {
    loop {
        let subchunk_id = parse_str(&buf, offset, 4);

        if subchunk_id == "data" {
            return Ok(());
        } else if *offset >= buf.len() {
            bail!("data chunk not found");
        }

        let size = parse_u32(&buf, offset) as usize;
        *offset += size;
    }
}

fn parse_u32(buf: &[u8], offset: &mut usize) -> u32 {
    let num = u32::from_le_bytes([
        buf[*offset],
        buf[*offset + 1],
        buf[*offset + 2],
        buf[*offset + 3],
    ]);

    *offset += 4;
    num
}

fn parse_u16(buf: &[u8], offset: &mut usize) -> u16 {
    let num = u16::from_le_bytes([buf[*offset], buf[*offset + 1]]);

    *offset += 2;
    num
}

fn parse_i16(buf: &[u8], offset: &mut usize) -> i16 {
    let num = i16::from_le_bytes([buf[*offset], buf[*offset + 1]]);

    *offset += 2;
    num
}

fn parse_str<'a>(buf: &'a [u8], offset: &mut usize, len: usize) -> &'a str {
    let str = std::str::from_utf8(&buf[*offset..(*offset + len)]).unwrap();
    *offset += len;

    str
}

fn compare_str_bytes(buf: &[u8], offset: &mut usize, string: &str) -> bool {
    let res = &buf[*offset..(*offset + string.len())] == string.as_bytes();
    *offset += string.len();
    res
}
