
use encoding_rs as enc;
use crate::Transcoder;

pub struct TranscodingReader<R: std::io::Read> {
    reader: R,
    bytes_to_guess: usize,
    non_ascii_to_guess: usize,
    non_text_threshold: u8,
    buffer: Vec<u8>,
    unread_buffer: Vec<u8>,
    unwritten_buffer: Vec<u8>,
    transcoder: Transcoder,
    had_replacement_or_unmappable: bool,
    transcode_done: bool,
    eof: bool,
}

impl <R: std::io::Read> TranscodingReader<R> {
    pub fn new(reader: R, src_encoding: Option<&'static enc::Encoding>, dst_encoding: &'static enc::Encoding) -> Self {
        return TranscodingReader {
            bytes_to_guess: 1024,
            non_ascii_to_guess: 100,
            non_text_threshold: 0,
            reader,
            buffer: vec![0u8; 8*1024],
            unread_buffer: vec![],
            unwritten_buffer: vec![],
            transcoder: Transcoder::new(src_encoding, dst_encoding),
            had_replacement_or_unmappable: false,
            transcode_done: false,
            eof: false,
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

    pub fn guess(self: &mut Self)
        -> std::io::Result<(Option<&'static enc::Encoding>, usize)> {
        // TODO guess done
        let src = &mut vec![0u8; self.bytes_to_guess];
        let buf_minus1 =src.len()-1;
        let first = self.reader.read(&mut src[..buf_minus1])?;
        if first == 0 {
            self.eof = true;
            self.transcode_done = true;
            return Ok((None, 0));
        }
        let second = self.reader.read(&mut src[buf_minus1..])?;
        self.eof = second == 0;
        let n = first +second;
        // TODO new transcoder
        let rslt = self.transcoder.guess_and_transcode(&src[..n], &mut self.buffer, self.non_ascii_to_guess, self.non_text_threshold, self.eof);
        let (guessed_enc, coder_result, num_read, num_written, has_replacement) = rslt;
        self.unread_buffer = src[num_read..].to_vec();
        self.unwritten_buffer = self.buffer[..num_written].to_vec();
        self.transcode_done = (coder_result == enc::CoderResult::InputEmpty) && self.eof;
        self.had_replacement_or_unmappable = has_replacement;
        return Ok((guessed_enc, num_read));
    }

    fn copy_from_unwritten_buffer(self: &mut Self, buffer: &mut [u8]) -> usize{
        let min = std::cmp::min(buffer.len(), self.unwritten_buffer.len());
        buffer[..min].copy_from_slice(&self.unwritten_buffer[..min]);
        self.unwritten_buffer = self.unwritten_buffer[min..].to_vec();
        return min;
    }

    fn transcode(self: &mut Self, buffer: &mut[u8], from_unread: bool) -> usize {
        let src = if from_unread {
            &mut self.unread_buffer
        } else {
            &mut self.buffer
        };

        if src.len() == 0 && !self.eof { // encoding_rs unable to handle unnecessary calls well, so let's skip them
            return 0;
        }

        if buffer.len() > 16 { // buffer has enough bytes for encoding_rs to write output
            let rslt = self.transcoder.transcode(src, buffer, self.eof);
            let (coder_result, num_read, num_written, has_replacement) = rslt;
            self.unread_buffer = src[num_read..].to_vec();
            self.had_replacement_or_unmappable = self.had_replacement_or_unmappable || has_replacement;
            self.transcode_done = (coder_result == enc::CoderResult::InputEmpty) && self.eof;
            if num_written > 0 {
                return num_written;
            }
        } else { // if the buffer is insufficient, let's create a buffer by ourselves
            let write_buffer = &mut [0u8; 8*1024];
            let rslt = self.transcoder.transcode(src, write_buffer, self.eof);
            let (coder_result, num_read, num_written, has_replacement) = rslt;
            self.unread_buffer = src[num_read..].to_vec();
            self.unwritten_buffer = write_buffer[..num_written].to_vec();
            self.had_replacement_or_unmappable = self.had_replacement_or_unmappable || has_replacement;
            self.transcode_done = (coder_result == enc::CoderResult::InputEmpty) && self.eof;
            if num_written > 0 {
                let n = self.copy_from_unwritten_buffer(buffer);
                return n;
            }
        }

        return 0;
    }
}

impl <R: std::io::Read> std::io::Read for TranscodingReader<R> {

    fn read(self: &mut Self, buffer: &mut [u8]) -> std::io::Result<usize> {

        if buffer.len() == 0 {
            return Ok(0);
        }

        if self.unwritten_buffer.len() > 0 {
            let num_written = self.copy_from_unwritten_buffer(buffer);
            return Ok(num_written);
        }

        if self.transcode_done {
            return Ok(0);
        }

        if self.unread_buffer.len() > 0 {
            let num_written = self.transcode(buffer, true);
            if num_written > 0 {
                return Ok(num_written);
            }
        }

        loop {
            let n = self.reader.read(&mut self.buffer)?;
            self.eof = n == 0;
            let num_written = self.transcode(buffer, false);
            return Ok(num_written);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::path;
    use super::*;

    // macro_rules! test_reader {
    //     ($name:ident, $input_file:expr, $expected_file:expr, $enc:expr, $read_buff_size:expr) => {
    //         #[test]
    //         fn $name() {
    //             let test_data = path::Path::new("../test_data");
    //             let ifile_handle = &mut std::fs::File::open(test_data.join($input_file)).unwrap();
    //             let enc = enc::Encoding::for_label($enc.as_bytes());
    //             let t = &mut TranscodingReader::new(ifile_handle, None, enc.unwrap())
    //                     .buffer_size(128)
    //                     .bytes_to_guess(256);
    //             let mut buff = vec![0u8; $read_buff_size];
    //             t.guess().unwrap();
    //             let n = t.read_to_end(&mut buff).unwrap();
    //             let efile_handle = &mut std::fs::File::open(test_data.join($expected_file)).unwrap();
    //             let expected_string = &mut Vec::new();
    //             efile_handle.read_to_end(expected_string).unwrap();
    //             assert_eq!(&expected_string[..n], &buff[..n]);
    //         }
    //     };
    // }

    macro_rules! test_reader {
        ($name:ident, $input_file:expr, $expected_file:expr, $enc:expr) => {
            #[test]
            fn $name() {
                let test_data = path::Path::new("../test_data");
                let ifile_handle = &mut std::fs::File::open(test_data.join($input_file)).unwrap();
                let enc = enc::Encoding::for_label($enc.as_bytes());
                let t = &mut TranscodingReader::new(ifile_handle, None, enc.unwrap());
                let mut buff = Vec::new();
                t.guess().unwrap();
                t.read_to_end(&mut buff).unwrap();
                let efile_handle = &mut std::fs::File::open(test_data.join($expected_file)).unwrap();
                let mut expected_string = Vec::new();
                efile_handle.read_to_end(&mut expected_string).unwrap();
                // assert!(expected_string == buff);
            }
        };
    }

    test_reader!(reader_sjis_utf8        , "sjis_ja.txt"         , "utf8_ja.txt"     , "utf8");

    test_reader!(reader_utf8_euckr       , "utf8_ko.txt"     , "euc-kr_ko.txt"       , "euc-kr");

//    test_guess!(guess_utf16le_utf8     , "utf16le_BOM_th.txt"  , "utf8_th.txt"     , "utf8");
//    test_guess!(guess_utf16be_utf8     , "utf16be_BOM_th.txt"  , "utf8_th.txt"     , "utf8");
//    test_guess!(guess_sjis_utf8        , "sjis_ja.txt"         , "utf8_ja.txt"     , "utf8");
//    test_guess!(guess_eucjp_utf8       , "euc-jp_ja.txt"       , "utf8_ja.txt"     , "utf8");
//    test_guess!(guess_iso2022jp_utf8   , "iso-2022-jp_ja.txt"  , "utf8_ja.txt"     , "utf8");
//    test_guess!(guess_big5_utf8        , "big5_zh_CHT.txt"     , "utf8_zh_CHT.txt" , "utf8");
//    test_guess!(guess_gbk_utf8         , "gbk_zh_CHS.txt"      , "utf8_zh_CHS.txt" , "utf8");
//    test_guess!(guess_gb18030_utf8     , "gb18030_zh_CHS.txt"  , "utf8_zh_CHS.txt" , "utf8");
//    test_guess!(guess_euckr_utf8       , "euc-kr_ko.txt"       , "utf8_ko.txt"     , "utf8");
//    test_guess!(guess_koi8r_utf8       , "koi8-r_ru.txt"       , "utf8_ru.txt"     , "utf8");
//    test_guess!(guess_windows1252_utf8 , "windows-1252_es.txt" , "utf8_es.txt"     , "utf8");
//
//    test_guess!(guess_utf8_utf16le     , "utf8_th.txt"     , "utf16le_th.txt"      , "utf-16le"     );
//    test_guess!(guess_utf8_utf16be     , "utf8_th.txt"     , "utf16be_th.txt"      , "utf-16be"     );
//    test_guess!(guess_utf8_sjis        , "utf8_ja.txt"     , "sjis_ja.txt"         , "sjis"         );
//    test_guess!(guess_utf8_eucjp       , "utf8_ja.txt"     , "euc-jp_ja.txt"       , "euc-jp"       );
//    test_guess!(guess_utf8_iso2022jp   , "utf8_ja.txt"     , "iso-2022-jp_ja.txt"  , "iso-2022-jp"  );
//    test_guess!(guess_utf8_big5        , "utf8_zh_CHT.txt" , "big5_zh_CHT.txt"     , "big5"         );
//    test_guess!(guess_utf8_gbk         , "utf8_zh_CHS.txt" , "gbk_zh_CHS.txt"      , "gbk"          );
//    test_guess!(guess_utf8_gb18030     , "utf8_zh_CHS.txt" , "gb18030_zh_CHS.txt"  , "gb18030"      );
//    test_guess!(guess_utf8_euckr       , "utf8_ko.txt"     , "euc-kr_ko.txt"       , "euc-kr"       );
//    test_guess!(guess_utf8_koi8r       , "utf8_ru.txt"     , "koi8-r_ru.txt"       , "koi8-r"       );
//    test_guess!(guess_utf8_windows1252 , "utf8_es.txt"     , "windows-1252_es.txt" , "windows-1252" );
}

