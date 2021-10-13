use encoding_rs as enc;
use chardetng as cd;
use std::str;

use crate::constants;


pub struct GuessResult {
    pub num_fed: usize,
    pub eof: bool,
}
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
            Some(dst_encoding.new_encoder())
        } else {
            None
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
    pub fn transcode(self: &mut Self, src: & [u8], dst: & mut [u8], last: bool) -> (enc::CoderResult, usize, usize) {
        let is_encoder_utf8 = self.dst_encoding == enc::UTF_8;
        let decoder = self.decoder.expect("transcode() should be called after decoder being detected.");
        if self.dst_encoding == enc::UTF_8 {
            let (result, num_decoder_read, num_decoder_written, _) =
                decoder.decode_to_utf8(src, dst, last);
            return (result, num_decoder_read, num_decoder_written);
        // } else if self.dst_encoding == enc::UTF_16BE || self.dst_encoding == enc::UTF_16LE {
        //     let (result, num_decoder_read, num_decoder_written, _) =
        //         decoder.decode_to_utf16(src, dst as &mut [u16], last);
        //     return (result, num_decoder_read, num_decoder_written);
        } else {
            let encoder = self.encoder.unwrap();
            let (decoder_result, num_decoder_read, num_decoder_written, _) =
                decoder.decode_to_utf8(src, &mut self.decode_buffer, last);
            self.unencoded_bytes.append(&mut self.decode_buffer[..num_decoder_written].to_vec());
            let encoder_input = unsafe {
                str::from_utf8_unchecked(&self.unencoded_bytes)
            };
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

    pub fn guess_and_transcode(self: &mut Self, src: &mut [u8], dst: & mut [u8], eof: bool, num_non_aschii: usize)
        -> Result<(enc::CoderResult, usize, usize), String> {

        // BOM sniffing

        // guess BOMless encodings
        let num_read = src.len();
        let mut det = cd::EncodingDetector::new();
        let mut aschii_cnt = 0;
        let mut num_fed = 0;
        let mut exhausted = num_read == num_fed;
        for b in src.iter() {
            num_fed+=1;
            exhausted = num_read == num_fed;
            if self.detector.feed(&[*b], eof && exhausted) {
                aschii_cnt+=1;
                if aschii_cnt > num_non_aschii {
                    break;
                }
            }
        }
        let top_level_domain = None;
        let allow_utf8 = true;
        let decoder = self.detector.guess(top_level_domain, allow_utf8).new_decoder();
        let guess_result = &mut GuessResult {
            num_fed,
            eof: eof && exhausted,
        };
        let (coder_result, decoder_read, decoder_written, auto_detection_failed)
            = Transcoder::try_transcode(self, &mut decoder, num_fed, eof, src, dst)?;
        self.decoder = Some(decoder);
        return Ok((coder_result, decoder_read, decoder_written));
    }


    fn try_transcode(self: &mut Self, decoder: &mut enc::Decoder, num_fed: usize, eof: bool, src: &[u8], dst: & mut [u8])
        -> Result<(enc::CoderResult, usize, usize, bool), String> {
        let (decoder_result, num_decoder_read, num_decoder_written, _) = decoder.decode_to_utf8(&src[..num_fed], &mut self.decode_buffer, eof);
        let decode_buffer_str = unsafe{
            str::from_utf8_unchecked_mut(&mut self.decode_buffer[..num_decoder_written])
        };
        let mut non_text_cnt = 0;
        for s in decode_buffer_str.chars() {
            if let Ok(_) = constants::NON_TEXTS_FREQUENT.binary_search(&s) {
                non_text_cnt+=1;
                continue;
            }
            if let Ok(_) = constants::NON_TEXTS.binary_search(&s) {
                non_text_cnt+=1;
            }
        }
        let auto_detection_failed = 1 < (non_text_cnt / decode_buffer_str.chars().count() );
        if auto_detection_failed {
            return Err("Auto-detection seems to fail.".into());
        }
        // if false {
        //     // utf16
        // }
        self.unencoded_bytes.append(&mut self.decode_buffer[..num_decoder_written].to_vec());
        let encoder_input = unsafe {
            str::from_utf8_unchecked(&self.unencoded_bytes)
        };
        let encoder = self.encoder.unwrap();
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

    pub fn src_encoding(self: &Self) -> Option<&'static enc::Encoding> {
        return self.src_encoding;
    }

    pub fn dst_encoding(self: &Self) -> &'static enc::Encoding {
        return self.dst_encoding;
    }
}


#[cfg(test)]
mod tests {
    macro_rules! test_trans_simple {
        ($name:ident, $enc:expr, $dec:expr, $srcbytes:expr, $dst:expr) => {
            #[test]
            fn $name() {
                let dec = &mut super::enc::Encoding::for_label($dec.as_bytes()).unwrap().new_decoder();
                let enc = &mut super::enc::Encoding::for_label($enc.as_bytes()).unwrap().new_encoder();
                let mut t = super::Transcoder::new(dec, enc);
                let output = &mut [0u8; 4];
                let (_,_,written) = t.transcode($srcbytes, output, true);
                // assert_eq!($dst, &output[..written]);
            }
        };
    }

    // This isn't exhaustive obviously, but it lets us test base level support.
    // test_trans_simple!(trans_simple_utf8       ,        "utf-8"   ,          "utf-8" , b"\xD0\x96" , b"\xD0\x96"); // Ж
    test_trans_simple!(trans_simple_utf16le    ,     "utf-16le"   ,       "utf-16le" , b"\x16\x04" , b"\x16\x04"); // Ж
    // test_trans_simple!(trans_simple_utf16be    ,     "utf-16be"   ,       "utf-16be" , b"\x04\x16" , b"\x04\x16"); // Ж
    // test_trans_simple!(trans_simple_chinese    ,     "chinese"    ,        "chinese" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    // test_trans_simple!(trans_simple_korean     ,      "korean"    ,         "korean" , b"\xAC\xA8" , b"\xAC\xA8"); // Ж
    // test_trans_simple!(trans_simple_big5_hkscs ,  "big5-hkscs"    ,     "big5-hkscs" , b"\xC7\xFA" , b"\xC7\xFA"); // Ж
    // test_trans_simple!(trans_simple_gbk        ,         "gbk"    ,            "gbk" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    // test_trans_simple!(trans_simple_sjis       ,        "sjis"    ,           "sjis" , b"\x84\x47" , b"\x84\x47"); // Ж
    // test_trans_simple!(trans_simple_eucjp      ,       "euc-jp"   ,         "euc-jp" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    // test_trans_simple!(trans_simple_latin1     ,      "latin1"    ,         "latin1" , b"\xA9"     , b"\xA9"    ); // ©
}
