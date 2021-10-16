use encoding_rs as enc;
use chardetng as cd;
use std::str;

use crate::constants;


pub struct Transcoder {
    src_encoding: Option<&'static enc::Encoding>,
    dst_encoding: &'static enc::Encoding,
    decoder: Option<enc::Decoder>,
    encoder: Option<enc::Encoder>,
    decode_buffer: Vec<u8>,
    unencoded_bytes: Vec<u8>,
    detector: cd::EncodingDetector
}

impl<'a> Transcoder {
    pub fn new(src_encoding: Option<&'static enc::Encoding>, dst_encoding: &'static enc::Encoding) -> Self {
        return Transcoder::new_with_buff_size(src_encoding, dst_encoding, 8*1024).unwrap();
    }
    pub fn new_with_buff_size(src_encoding: Option<&'static enc::Encoding>, dst_encoding: &'static enc::Encoding, size: usize)
        -> Result<Self, String> {
        if size < 4 { // at least 3 bytes are required for encoding_rs to write a valid UTF-8 character.
            let msg = format!("Buffer of size {} is too small", size);
            return Err(msg);
        }
        let encoder = if dst_encoding == enc::UTF_16BE || dst_encoding == enc::UTF_16LE {
            None
        } else {
            Some(dst_encoding.new_encoder())
        };
        return Ok(Transcoder {
            src_encoding,
            dst_encoding,
            decoder: src_encoding.map(|s| s.new_decoder()),
            encoder,
            decode_buffer: vec![0u8;8*1024],
            unencoded_bytes: Vec::with_capacity(8*1024),
            detector: cd::EncodingDetector::new(),
        });
    }
    pub fn transcode(self: &mut Self, src: &[u8], dst: &mut [u8], last: bool) -> (enc::CoderResult, usize, usize) {
        let decoder = self.decoder.as_mut().expect("transcode() should be called after the source encoding is detected.");
        if self.dst_encoding == enc::UTF_8 {
            println!("no encoding");
            let (result, num_decoder_read, num_decoder_written, _) = decoder.decode_to_utf8(src, dst, last);
            return (result, num_decoder_read, num_decoder_written);
        } else if self.dst_encoding == enc::UTF_16BE || self.dst_encoding == enc::UTF_16LE {
            let dst_u16 = &mut vec![0u16; dst.len()/2];
            let (result, num_decoder_read, num_decoder_written, _) =
                decoder.decode_to_utf16(src, dst_u16, last);
            Transcoder::u16_to_u8(dst_u16, dst, num_decoder_written, self.dst_encoding == enc::UTF_16BE);
            return (result, num_decoder_read, num_decoder_written*2);
        } else {
            let (decoder_result, num_decoder_read, num_decoder_written, _) =
                decoder.decode_to_utf8(src, &mut self.decode_buffer, last);
            self.unencoded_bytes.append(&mut self.decode_buffer[..num_decoder_written].to_vec());
            let encoder_input = unsafe {
                str::from_utf8_unchecked(&self.unencoded_bytes)
            };
            let encoder = self.encoder.as_mut().unwrap();
            println!("decoded: {:?}", encoder_input);
            let (encoder_result, num_encoder_read, num_encoder_written, _) =
                encoder.encode_from_utf8(encoder_input, dst, last);
            self.unencoded_bytes = self.unencoded_bytes[num_encoder_read..].to_vec();
            let result = if decoder_result == enc::CoderResult::InputEmpty && encoder_result == enc::CoderResult::InputEmpty {
                enc::CoderResult::InputEmpty
            } else {
                enc::CoderResult::OutputFull
            };
            return (result, num_decoder_read, num_encoder_written);
        }
    }

    pub fn guess_and_transcode(self: &mut Self, src: &mut [u8], dst: & mut [u8], chars_to_guess: usize, non_text_limit: u8, eof: bool)
        -> Result<(enc::CoderResult, usize, usize), String> {

        // guess the encoding and get a decoder
        let mut decoder = match enc::Encoding::for_bom(src) { // BOM sniffing
            Some(found_bom) => {
                let (encoding,_) = found_bom;
                encoding.new_decoder()
            },
            None => { // guess BOMless encodings
                let num_read = src.len();
                let mut non_ascii_cnt = 0;
                let mut num_fed = 0;
                let mut exhausted;
                for b in src.iter() {
                    num_fed+=1;
                    exhausted = num_read == num_fed;
                    let is_non_ascii = self.detector.feed(&[*b], eof && exhausted);
                    let is_non_text = Transcoder::is_non_text(&(*b as char));
                    if is_non_ascii || is_non_text {
                        non_ascii_cnt+=1;
                        if non_ascii_cnt > chars_to_guess {
                            break;
                        }
                    }
                }
                println!("non ascii count: {:?}", non_ascii_cnt);
                println!("number of fed bytes: {:?}", num_fed);
                let top_level_domain = None;
                let allow_utf8 = true;
                self.detector.guess(top_level_domain, allow_utf8).new_decoder()
            },
        };
        println!("guessed decoder: {:?}", decoder.encoding());
        let (coder_result, decoder_read, decoder_written)
            = Transcoder::try_transcode(self, &mut decoder, eof, src, dst, non_text_limit)?;
        self.src_encoding = Some(decoder.encoding());
        self.decoder = Some(decoder);
        return Ok((coder_result, decoder_read, decoder_written));
    }


    fn try_transcode(self: &mut Self, decoder: &mut enc::Decoder, eof: bool, src: &[u8], dst: &mut [u8], non_text_limit: u8)
        -> Result<(enc::CoderResult, usize, usize), String> {
        let decode_buffer = if self.dst_encoding == enc::UTF_8 {
            &mut (*dst)
        } else {
            &mut self.decode_buffer
        };
        let (decoder_result, num_decoder_read, num_decoder_written, _) = decoder.decode_to_utf8(src, decode_buffer, eof);
        let decode_buffer_str = unsafe{
            str::from_utf8_unchecked_mut(&mut decode_buffer[..num_decoder_written])
        };
        let mut non_text_cnt = 0;
        for c in decode_buffer_str.chars() {
            if Transcoder::is_non_text(&c) {
                println!("non text: {:?}", c);
                non_text_cnt+=1;
            }
        }
        let auto_detection_failed = (non_text_limit as usize) < (non_text_cnt * 100 / decode_buffer_str.chars().count() );
        if auto_detection_failed {
            return Err("Auto-detection seems to fail.".into());
        }
        if self.dst_encoding == enc::UTF_8 {
            return Ok((decoder_result, num_decoder_read, num_decoder_written));
        }
        if self.dst_encoding == enc::UTF_16BE || self.dst_encoding == enc::UTF_16LE {
            println!("decode_to_utf16");
            self.decoder = Some(decoder.encoding().new_decoder()); // the decoder was used once to check the guess result, so we need a new one.
            let new_decoder = self.decoder.as_mut().unwrap();
            let dst_u16 = &mut vec![0u16; dst.len()/2];
            let (result, num_decoder_read, num_decoder_written, _) =
                new_decoder.decode_to_utf16(src, dst_u16, eof);
            Transcoder::u16_to_u8(dst_u16, dst, num_decoder_written, self.dst_encoding == enc::UTF_16BE);
            return Ok((result, num_decoder_read, num_decoder_written*2));
        } else {
            self.unencoded_bytes.append(&mut decode_buffer[..num_decoder_written].to_vec());
            let encoder_input = unsafe {
                str::from_utf8_unchecked(&self.unencoded_bytes)
            };
            let encoder = self.encoder.as_mut().unwrap();
            let (encoder_result, num_encoder_read, num_encoder_written, _) =
                encoder.encode_from_utf8(encoder_input, dst, eof);
            self.unencoded_bytes = self.unencoded_bytes[num_encoder_read..].to_vec();
            let coder_result = if decoder_result == enc::CoderResult::InputEmpty && encoder_result == enc::CoderResult::InputEmpty {
                enc::CoderResult::InputEmpty
            } else {
                enc::CoderResult::OutputFull
            };
            return Ok((coder_result, num_decoder_read, num_encoder_written));
        }
    }


    pub fn is_non_text(c: &char) -> bool {
        if let Ok(_) = constants::NON_TEXTS_FREQUENT.binary_search(&c) {
            return true;
        }
        if let Ok(_) = constants::NON_TEXTS.binary_search(&c) {
            return true;
        }
        return false;
    }

    pub fn src_encoding(self: &Self) -> Option<&'static enc::Encoding> {
        return self.src_encoding;
    }

    pub fn dst_encoding(self: &Self) -> &'static enc::Encoding {
        return self.dst_encoding;
    }

    fn u16_to_u8(src: &[u16], dst: &mut [u8], src_length: usize, is_be: bool) {
        log::debug!("{}",src_length);
        println!("{}",src_length);
        let to_bytes = if is_be {
            |src| u16::to_be_bytes(src)
        } else {
            |src| u16::to_le_bytes(src)
        };
        for i in 0..src_length {
            let bytes = to_bytes(src[i]);
            dst[i*2] = bytes[0];
            dst[i*2+1] = bytes[1];
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::path;

    macro_rules! test_guess {
        ($name:ident, $input_file:expr, $expected_file:expr, $enc:expr) => {
            #[test]
            fn $name() {
                let test_data = path::Path::new("test_data");
                let ifile_handle = &mut std::fs::File::open(test_data.join($input_file)).unwrap();
                let input_bytes = &mut [0u8; 500];
                ifile_handle.read(input_bytes).unwrap();
                let enc = super::enc::Encoding::for_label($enc.as_bytes());
                let t = &mut super::Transcoder::new(None, enc.unwrap());
                let output_bytes = &mut [0u8; 1024];
                // println!("{:x?}", &input_bytes[..15]);
                match t.guess_and_transcode(input_bytes, output_bytes, 100, 5, false) {
                    Ok((_, _, num_written)) => {
                        let test_data = path::Path::new("test_data");
                        let efile_handle = &mut std::fs::File::open(test_data.join($expected_file)).unwrap();
                        let expected_string = &mut Vec::new();
                        efile_handle.read_to_end(expected_string).unwrap();
                        assert_eq!(&expected_string[..num_written], &output_bytes[..num_written]);
                    },
                    Err(err) => panic!("{:?}",err),
                }
            }
        };
    }

    test_guess!(guess_utf16le_utf8     , "utf16le_BOM_th.txt"  , "utf8_th.txt"     , "utf8");
    test_guess!(guess_utf16be_utf8     , "utf16be_BOM_th.txt"  , "utf8_th.txt"     , "utf8");
    test_guess!(guess_sjis_utf8        , "sjis_ja.txt"         , "utf8_ja.txt"     , "utf8");
    test_guess!(guess_eucjp_utf8       , "euc-jp_ja.txt"       , "utf8_ja.txt"     , "utf8");
    test_guess!(guess_iso2022jp_utf8   , "iso-2022-jp_ja.txt"  , "utf8_ja.txt"     , "utf8");
    test_guess!(guess_big5_utf8        , "big5_zh_CHT.txt"     , "utf8_zh_CHT.txt" , "utf8");
    test_guess!(guess_gbk_utf8         , "gbk_zh_CHS.txt"      , "utf8_zh_CHS.txt" , "utf8");
    test_guess!(guess_gb18030_utf8     , "gb18030_zh_CHS.txt"  , "utf8_zh_CHS.txt" , "utf8");
    test_guess!(guess_euckr_utf8       , "euc-kr_ko.txt"       , "utf8_ko.txt"     , "utf8");
    test_guess!(guess_koi8r_utf8       , "koi8-r_ru.txt"       , "utf8_ru.txt"     , "utf8");
    test_guess!(guess_windows1252_utf8 , "windows-1252_es.txt" , "utf8_es.txt"     , "utf8");

    test_guess!(guess_utf8_utf16le     , "utf8_th.txt"     , "utf16le_th.txt"      , "utf-16le"     );
    test_guess!(guess_utf8_utf16be     , "utf8_th.txt"     , "utf16be_th.txt"      , "utf-16be"     );
    test_guess!(guess_utf8_sjis        , "utf8_ja.txt"     , "sjis_ja.txt"         , "sjis"         );
    test_guess!(guess_utf8_eucjp       , "utf8_ja.txt"     , "euc-jp_ja.txt"       , "euc-jp"       );
    test_guess!(guess_utf8_iso2022jp   , "utf8_ja.txt"     , "iso-2022-jp_ja.txt"  , "iso-2022-jp"  );
    test_guess!(guess_utf8_big5        , "utf8_zh_CHT.txt" , "big5_zh_CHT.txt"     , "big5"         );
    test_guess!(guess_utf8_gbk         , "utf8_zh_CHS.txt" , "gbk_zh_CHS.txt"      , "gbk"          );
    test_guess!(guess_utf8_gb18030     , "utf8_zh_CHS.txt" , "gb18030_zh_CHS.txt"  , "gb18030"      );
    test_guess!(guess_utf8_euckr       , "utf8_ko.txt"     , "euc-kr_ko.txt"       , "euc-kr"       );
    test_guess!(guess_utf8_koi8r       , "utf8_ru.txt"     , "koi8-r_ru.txt"       , "koi8-r"       );
    test_guess!(guess_utf8_windows1252 , "utf8_es.txt"     , "windows-1252_es.txt" , "windows-1252" );

    #[test]
    fn test_guess_error() {
        let file_handle = &mut std::fs::File::open("test_data/binary.jpeg").unwrap();
        let input = &mut [0u8; 500];
        file_handle.read(input).unwrap();
        let enc = super::enc::Encoding::for_label("utf-8".as_bytes());
        let t = &mut super::Transcoder::new(None, enc.unwrap());
        let output = &mut [0u8; 1024];
        match t.guess_and_transcode(input, output, 100, 0, false) {
            Ok(ok) => panic!("{:?}", ok),
            Err(_) => (),
        }
    }

    macro_rules! transcode_test {
        ($name:ident, $dec:expr, $enc:expr, $srcbytes:expr, $dst:expr) => {
            #[test]
            fn $name() {
                let dec = super::enc::Encoding::for_label($dec.as_bytes());
                println!("decoder: {:?}", dec.unwrap());
                let enc = super::enc::Encoding::for_label($enc.as_bytes());
                println!("encoder: {:?}", enc.unwrap());
                let mut t = super::Transcoder::new(dec, enc.unwrap());
                // let output = &mut [0u8; 14]; // encoder seems to need at least 14 bytes
                let output = &mut [0u8; 140]; // encoder seems to need at least 14 bytes
                let (_,_,written) = t.transcode($srcbytes, output, false);
                println!("written: {}",written);
                assert_eq!($dst, &output[..written]);
            }
        };
    }

    // This isn't exhaustive obviously, but it lets us test base level support.
    transcode_test!(trans_same_utf8       ,        "utf-8" ,          "utf-8" , b"\xD0\x96" , b"\xD0\x96"); // Ж
    transcode_test!(trans_same_utf16le    ,     "utf-16le" ,       "utf-16le" , b"\x16\x04" , b"\x16\x04"); // Ж
    transcode_test!(trans_same_utf16be    ,     "utf-16be" ,       "utf-16be" , b"\x04\x16" , b"\x04\x16"); // Ж
    transcode_test!(trans_same_chinese    ,     "chinese"  ,        "chinese" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    transcode_test!(trans_same_korean     ,      "korean"  ,         "korean" , b"\xAC\xA8" , b"\xAC\xA8"); // Ж
    transcode_test!(trans_same_big5_hkscs ,  "big5-hkscs"  ,     "big5-hkscs" , b"\xC7\xFA" , b"\xC7\xFA"); // Ж
    transcode_test!(trans_same_gbk        ,         "gbk"  ,            "gbk" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    transcode_test!(trans_same_sjis       ,        "sjis"  ,           "sjis" , b"\x84\x47" , b"\x84\x47"); // Ж
    transcode_test!(trans_same_eucjp      ,       "euc-jp" ,         "euc-jp" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    transcode_test!(trans_same_latin1     ,      "latin1"  ,         "latin1" , b"\xA9"     , b"\xA9"    ); // ©

    transcode_test!(trans_diff_utf8_utf16le       ,        "utf-8" ,       "utf-16le" , b"\xD0\x96"     , b"\x16\x04"); // Ж
    transcode_test!(trans_diff_utf16le_utf16be    ,     "utf-16le" ,       "utf-16be" , b"\x16\x04"     , b"\x04\x16"); // Ж
    transcode_test!(trans_diff_utf16be_chinese    ,     "utf-16be" ,        "chinese" , b"\x04\x16"     , b"\xA7\xA8"); // Ж
    transcode_test!(trans_diff_chinese_korean     ,     "chinese"  ,         "korean" , b"\xA7\xA8"     , b"\xAC\xA8"); // Ж
    transcode_test!(trans_diff_korean_big5_hkscs  ,      "korean"  ,     "big5-hkscs" , b"\xAC\xA8"     , b"\xC7\xFA"); // Ж
    transcode_test!(trans_diff_big5_hkscs_gbk     ,  "big5-hkscs"  ,            "gbk" , b"\xC7\xFA"     , b"\xA7\xA8"); // Ж
    transcode_test!(trans_diff_gbk_sjis           ,         "gbk"  ,           "sjis" , b"\xA7\xA8"     , b"\x84\x47"); // Ж
    transcode_test!(trans_diff_sjis_eucjp         ,        "sjis"  ,         "euc-jp" , b"\x84\x47"     , b"\xA7\xA8"); // Ж
    transcode_test!(trans_diff_eucjp_latin1       ,       "euc-jp" ,         "latin1" , b"\x8F\xA2\xED" , b"\xA9"    ); // ©
    transcode_test!(trans_diff_latin1_utf8        ,      "latin1"  ,          "utf-8" , b"\xA9"         , b"\xC2\xA9"); // ©
}
