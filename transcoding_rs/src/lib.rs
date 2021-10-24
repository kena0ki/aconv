//! This is a transcoding library.
//! Transcoding here means converting text encoding to another.
//!
//! There are two excellent crates [`chardetng`](https://github.com/hsivonen/chardetng) and [`encoding_rs`](https://github.com/hsivonen/encoding_rs).
//! `chardetng` is created for encoding detection and `encoding_rs` can be used for transcoding.
//! This library aims to transcode the easy and efficient way by combining these two crates.
//!
//!  Note: Supported encodings are the ones defined in [the Encoding Standard](https://encoding.spec.whatwg.org).  
//!
//!  Note: UTF-16 files are needed to have a BOM to be detected as the encoding.  
//!        This is because [`chardetng`](https://github.com/hsivonen/chardetng), on which this library depends, does not support UTF-16 and this library only added BOM sniffing to detect UTF-16.  
//!


mod transcoder;
mod constants;
mod i18n_reader;

pub use i18n_reader::I18nReaderEncodingDetector;
pub use i18n_reader::GuessResult;
pub use i18n_reader::I18nReader;
pub use transcoder::Transcoder;
pub use constants::ENCODINGS;

