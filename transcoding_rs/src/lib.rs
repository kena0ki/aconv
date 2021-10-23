//! This is a transcoding library.
//! Transcoding here means decoding the source texts and encoding them to the other encoding.
//!
//! There are two excellent crates [`chardetng`](https://github.com/hsivonen/chardetng) and [`encoding_rs`](https://github.com/hsivonen/encoding_rs).
//! `chardetng` is created for encoding detection and `encoding_rs` can be used for transcoding.
//! This library aims to transcode easy and efficient way by combining these two crates.
//!

mod transcoder;
mod constants;
mod i18n_reader;

pub use i18n_reader::I18nReaderEncodingDetector;
pub use i18n_reader::GuessResult;
pub use i18n_reader::I18nReader;
pub use transcoder::Transcoder;
pub use constants::ENCODINGS;

