
use encoding_rs as enc;
use crate::Transcoder;

pub struct I18nReaderFactory {
    bytes_to_guess: usize,
    non_ascii_to_guess: usize,
    non_text_threshold: u8,
    buffer: Vec<u8>,
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
    had_replacement_or_unmappable: bool,
    transcode_done: bool,
    eof: bool,
    no_transcoding_needed: bool,
}

pub enum GuessResult<R: std::io::Read> {
    InputEmpty,
    Success(I18nReader<R>, &'static enc::Encoding),
    Fail(I18nReader<R>),
}

impl I18nReaderFactory {
    pub fn new() -> Self {
        return Self {
            bytes_to_guess: 1024,
            non_ascii_to_guess: 100,
            non_text_threshold: 0,
            buffer: vec![0u8; 8*1024],
            read_buffer: vec![],
            write_buffer: vec![],
            had_replacement_or_unmappable: false,
            transcode_done: false,
            eof: false,
            no_transcoding_needed: false,
        };
    }

    pub fn buffer_size(mut self: Self, size: usize) -> Self {
        if size < 16 { // if the buffer is insufficient, let's ignore the specified size.
            return self;
        }
        self.buffer = vec![0u8; size];
        return self;
    }

    pub fn bytes_to_guess(mut self: Self, size: usize) -> Self {
        self.bytes_to_guess = size;
        return self;
    }

    pub fn non_text_threshold(mut self: Self, percent: u8) -> Self {
        self.non_text_threshold = percent;
        return self;
    }

    pub fn non_ascii_to_guess(mut self: Self, num: usize) -> Self {
        self.non_ascii_to_guess = num;
        return self;
    }

    pub fn guess_and_get<R>(self: Self, reader: R)
        -> std::io::Result<GuessResult<R>>
        where R: std::io::Read {
        return self.guess_and_get_with_dst_encoding(reader, enc::UTF_8);
    }

    pub fn guess_and_get_with_dst_encoding<R>(mut self: Self, mut reader: R, dst_encoding: &'static enc::Encoding)
        -> std::io::Result<GuessResult<R>>
        where R: std::io::Read {
        let read_buf = &mut vec![0u8; self.bytes_to_guess];
        let buf_minus1 =read_buf.len()-1;
        let first = reader.read(&mut read_buf[..buf_minus1])?;
        let is_empty = first == 0;
        if is_empty {
            self.eof = true;
            self.transcode_done = true;
            return Ok(GuessResult::InputEmpty);
        }
        let second = reader.read(&mut read_buf[buf_minus1..])?;
        self.eof = second == 0;
        let n = first +second;
        let src = &read_buf[..n];
        let mut transcoder = Transcoder::new(None, dst_encoding).buffer_size(self.buffer.len());
        let rslt = transcoder.guess_and_transcode(src, &mut self.buffer, self.non_ascii_to_guess, self.non_text_threshold, self.eof);
        let (guessed_enc_opt, coder_result, num_read, num_written, has_replacement) = rslt;
        self.no_transcoding_needed = guessed_enc_opt.is_none()
            || (guessed_enc_opt.is_some() && guessed_enc_opt.unwrap() == dst_encoding);
        if self.no_transcoding_needed {
            self.no_transcoding_needed = true;
            self.write_buffer = src.to_owned();
        } else {
            self.transcode_done = (coder_result == enc::CoderResult::InputEmpty) && self.eof;
            self.had_replacement_or_unmappable = has_replacement;
            self.read_buffer = src[num_read..].into();
            self.write_buffer = {
                if dst_encoding == enc::UTF_16BE && [0xFE,0xFF] != self.buffer[..2] {
                    [b"\xFE\xFF", &self.buffer[..num_written]].concat() // add a BOM
                } else if dst_encoding == enc::UTF_16LE && [0xFF,0xFE] != self.buffer[..2] {
                    [b"\xFF\xFE", &self.buffer[..num_written]].concat() // add a BOM
                } else{
                    self.buffer[..num_written].into()
                }
            };
        }
        if let Some(enc) = guessed_enc_opt {
            return Ok(GuessResult::Success(I18nReader::new_from_factory(reader, transcoder, self), enc));
        } else {
            return Ok(GuessResult::Fail(I18nReader::new_from_factory(reader, transcoder, self)));
        }
    }
}

pub struct I18nReader<R: std::io::Read> {
    reader: R,
    buffer: Vec<u8>,
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
    transcoder: Transcoder,
    had_replacement_or_unmappable: bool,
    transcode_done: bool,
    eof: bool,
    no_transcoding_needed: bool,
}

impl <R: std::io::Read> I18nReader<R> {

    pub fn new(reader: R, transcoder: Transcoder) -> Self {
        return Self {
            reader,
            buffer: vec![0u8; 8*1024],
            read_buffer: vec![],
            write_buffer: vec![],
            transcoder,
            had_replacement_or_unmappable: false,
            transcode_done: false,
            eof: false,
            no_transcoding_needed: false,
        };
    }

    pub fn new_from_factory(reader: R, transcoder: Transcoder, factory: I18nReaderFactory) -> Self {
        return Self {
            reader,
            buffer: factory.buffer,
            read_buffer: factory.read_buffer,
            write_buffer: factory.write_buffer,
            transcoder,
            had_replacement_or_unmappable: factory.had_replacement_or_unmappable,
            transcode_done: factory.transcode_done,
            eof: factory.eof,
            no_transcoding_needed: factory.no_transcoding_needed,
        };
    }

    fn copy_from_write_buffer_to(self: &mut Self, buffer: &mut [u8]) -> usize{
        let min = std::cmp::min(buffer.len(), self.write_buffer.len());
        buffer[..min].copy_from_slice(&self.write_buffer[..min]);
        self.write_buffer = self.write_buffer[min..].into();
        return min;
    }

    fn run_transcode(self: &mut Self, buffer: &mut[u8]) -> usize {
        let src = &mut self.read_buffer;

        if src.len() == 0 && !self.eof { // encoding_rs unable to handle unnecessary calls well, so let's skip them
            return 0;
        }

        if buffer.len() > 16 { // buffer has enough bytes for encoding_rs to write output
            let rslt = self.transcoder.transcode(src, buffer, self.eof);
            let (coder_result, num_read, num_written, has_replacement) = rslt;
            self.read_buffer = src[num_read..].into();
            self.had_replacement_or_unmappable = self.had_replacement_or_unmappable || has_replacement;
            self.transcode_done = (coder_result == enc::CoderResult::InputEmpty) && self.eof;
            if num_written > 0 {
                return num_written;
            }
        } else { // if the buffer is insufficient, let's create a buffer by ourselves
            let write_buffer = &mut [0u8; 8*1024];
            let rslt = self.transcoder.transcode(src, write_buffer, self.eof);
            let (coder_result, num_read, num_written, has_replacement) = rslt;
            self.read_buffer = src[num_read..].into();
            self.write_buffer = write_buffer[..num_written].into();
            self.had_replacement_or_unmappable = self.had_replacement_or_unmappable || has_replacement;
            self.transcode_done = (coder_result == enc::CoderResult::InputEmpty) && self.eof;
            if num_written > 0 {
                let n = self.copy_from_write_buffer_to(buffer);
                return n;
            }
        }

        return 0;
    }
}

impl <R: std::io::Read> std::io::Read for I18nReader<R> {

    fn read(self: &mut Self, buffer: &mut [u8]) -> std::io::Result<usize> {

        if buffer.len() == 0 {
            return Ok(0);
        }

        if self.write_buffer.len() > 0 {
            let num_written = self.copy_from_write_buffer_to(buffer);
            return Ok(num_written);
        }

        if self.no_transcoding_needed {
            let n = self.reader.read(buffer)?;
            return Ok(n);
        }

        if self.transcode_done {
            return Ok(0);
        }

        if self.read_buffer.len() > 0 {
            let num_written = self.run_transcode(buffer);
            if num_written > 0 {
                return Ok(num_written);
            }
        }

        let n = self.reader.read(&mut self.buffer)?;
        self.read_buffer = self.buffer[..n].into();
        self.eof = n == 0;
        let num_written = self.run_transcode(buffer);
        return Ok(num_written);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::path;
    use super::*;

    macro_rules! test_reader {
        ($name:ident, $input_file:expr, $expected_file:expr, $enc:expr) => {
            #[test]
            fn $name() {
                let test_data = path::Path::new("../test_data");
                let ifile_handle = &mut std::fs::File::open(test_data.join($input_file)).unwrap();
                let enc = enc::Encoding::for_label($enc.as_bytes());
                let f = I18nReaderFactory::new().bytes_to_guess(512);
                let r = f.guess_and_get_with_dst_encoding(ifile_handle, enc.unwrap()).unwrap();
                if let GuessResult::Success(mut reader, _) = r {
                    let mut buff = Vec::new();
                    reader.read_to_end(&mut buff).unwrap();
                    let efile_handle = &mut std::fs::File::open(test_data.join($expected_file)).unwrap();
                    let mut expected_string = Vec::new();
                    efile_handle.read_to_end(&mut expected_string).unwrap();
                    assert!(expected_string == buff);
                } else {
                    panic!();
                }
            }
        };
    }

    test_reader!(reader_sjis_utf8        , "sjis_ja.txt"         , "utf8_ja.txt"     , "utf8");

    test_reader!(reader_utf8_euckr       , "utf8_ko.txt"     , "euc-kr_ko.txt"       , "euc-kr");


    #[test]
    fn reader_small() {
        let src = b"\x83\x6E\x83\x8D\x81\x5B\x83\x8F\x81\x5B\x83\x8B\x83\x68";
        let f = I18nReaderFactory::new().buffer_size(15);
        let r = f.guess_and_get(src.as_ref()).unwrap();
        if let GuessResult::Success(mut reader, _) = r {
            let mut buff = [0u8; 4];
            let n = reader.read(&mut buff).unwrap();
            let mut buff2 = [0u8; 1024];
            let n2 = reader.read(&mut buff2).unwrap();
            assert_eq!("ハローワールド".as_bytes(), [&buff[..n],&buff2[..n2]].concat())
        } else {
            panic!();
        }
    }

    #[test]
    fn reader_fail() {
        let src = b"\x00\x00\x00\x00\x00\x00";
        let f = I18nReaderFactory::new().bytes_to_guess(512);
        let r = f.guess_and_get(src.as_ref()).unwrap();
        if let GuessResult::Fail(mut reader) = r {
            let mut buff = [0u8; 1024];
            let n = reader.read(&mut buff).unwrap();
            assert_eq!(b"\x00\x00\x00\x00\x00\x00"[..], buff[..n])
        } else {
            panic!();
        }
    }

    #[test]
    fn reader_empty() {
        let src = b"";
        let f = I18nReaderFactory::new();
        let r = f.guess_and_get(src.as_ref()).unwrap();
        if let GuessResult::InputEmpty = r {
        } else {
            panic!();
        }
    }

    #[test]
    fn reader_no_guess() {
        let src = b"\x83\x6E\x83\x8D\x81\x5B\x83\x8F\x81\x5B\x83\x8B\x83\x68";
        let t = Transcoder::new(Some(enc::SHIFT_JIS), enc::UTF_8);
        let mut reader = I18nReader::new(src.as_ref(), t);
        let mut buff = [0u8; 1024];
        let n = reader.read(&mut buff).unwrap();
        assert_eq!("ハローワールド".as_bytes(), &buff[..n])
    }
}

