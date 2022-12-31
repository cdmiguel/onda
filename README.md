# onda
Simple WAV file reader and writer written in Rust.

## Examples

```rust
// Read a WAV file
let wavdata = onda::read("foo.wav").unwrap();

// Write a WAV file
onda::write(wavdata.audiodata, wavdata.samplerate, "bar.wav").unwrap();
```