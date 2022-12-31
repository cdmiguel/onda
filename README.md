# onda
Simple WAV file reader and writer written in Rust.

## Examples

```rust
// Read a WAV file
let wavdata = onda::read_from_file("foo.wav").unwrap();

// Write a WAV file
onda::write_into_file(wavdata.audiodata, wavdata.samplerate, "bar.wav").unwrap();
```