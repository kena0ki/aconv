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
        let decoder = self.decoder.as_mut().expect("transcode() should be called after decoder being detected.");
        if self.dst_encoding == enc::UTF_8 {
            println!("no encode");
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

    pub fn guess_and_transcode(self: &mut Self, src: &mut [u8], dst: & mut [u8], eof: bool, num_non_aschii: usize)
        -> Result<(enc::CoderResult, usize, usize), String> {

        // BOM sniffing

        // guess BOMless encodings
        let num_read = src.len();
        let mut aschii_cnt = 0;
        let mut num_fed = 0;
        let mut exhausted;
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
        let mut decoder = self.detector.guess(top_level_domain, allow_utf8).new_decoder();
        let (coder_result, decoder_read, decoder_written)
            = Transcoder::try_transcode(self, &mut decoder, num_fed, eof, src, dst)?;
        self.decoder = Some(decoder);
        return Ok((coder_result, decoder_read, decoder_written));
    }


    fn try_transcode(self: &mut Self, decoder: &mut enc::Decoder, num_fed: usize, eof: bool, src: &[u8], dst: & mut [u8])
        -> Result<(enc::CoderResult, usize, usize), String> {
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
        if self.dst_encoding == enc::UTF_16BE || self.dst_encoding == enc::UTF_16LE {
            let dst_u16 = &mut vec![0u16; dst.len()/2];
            let (result, num_decoder_read, num_decoder_written, _) =
                decoder.decode_to_utf16(src, dst_u16, eof);
            Transcoder::u16_to_u8(dst_u16, dst, num_decoder_written, self.dst_encoding == enc::UTF_16BE);
            return Ok((result, num_decoder_read, num_decoder_written*2));
        } else {
            self.unencoded_bytes.append(&mut self.decode_buffer[..num_decoder_written].to_vec());
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
    macro_rules! test_trans_simple {
        ($name:ident, $dec:expr, $enc:expr, $srcbytes:expr, $dst:expr) => {
            #[test]
            fn $name() {
                let dec = super::enc::Encoding::for_label($dec.as_bytes());
                println!("decoder: {:?}", dec.unwrap());
                let enc = super::enc::Encoding::for_label($enc.as_bytes());
                println!("encoder: {:?}", enc.unwrap());
                let mut t = super::Transcoder::new(dec, enc.unwrap());
                let output = &mut [0u8; 14]; // encoder seems to need at least 14 bytes
                let (_,_,written) = t.transcode($srcbytes, output, true);
                println!("written: {}",written);
                assert_eq!($dst, &output[..written]);
            }
        };
    }

    // This isn't exhaustive obviously, but it lets us test base level support.
    test_trans_simple!(trans_same_utf8       ,        "utf-8"   ,          "utf-8" , b"\xD0\x96" , b"\xD0\x96"); // Ж
    test_trans_simple!(trans_same_utf16le    ,     "utf-16le"   ,       "utf-16le" , b"\x16\x04" , b"\x16\x04"); // Ж
    test_trans_simple!(trans_same_utf16be    ,     "utf-16be"   ,       "utf-16be" , b"\x04\x16" , b"\x04\x16"); // Ж
    test_trans_simple!(trans_same_chinese    ,     "chinese"    ,        "chinese" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_same_korean     ,      "korean"    ,         "korean" , b"\xAC\xA8" , b"\xAC\xA8"); // Ж
    test_trans_simple!(trans_same_big5_hkscs ,  "big5-hkscs"    ,     "big5-hkscs" , b"\xC7\xFA" , b"\xC7\xFA"); // Ж
    test_trans_simple!(trans_same_gbk        ,         "gbk"    ,            "gbk" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_same_sjis       ,        "sjis"    ,           "sjis" , b"\x84\x47" , b"\x84\x47"); // Ж
    test_trans_simple!(trans_same_eucjp      ,       "euc-jp"   ,         "euc-jp" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_same_latin1     ,      "latin1"    ,         "latin1" , b"\xA9"     , b"\xA9"    ); // ©

    test_trans_simple!(trans_diff_utf8_utf16le       ,        "utf-8"   ,       "utf-16le" , b"\xD0\x96"     , b"\x16\x04"); // Ж
    test_trans_simple!(trans_diff_utf16le_utf16be    ,     "utf-16le"   ,       "utf-16be" , b"\x16\x04"     , b"\x04\x16"); // Ж
    test_trans_simple!(trans_diff_utf16be_chinese    ,     "utf-16be"   ,        "chinese" , b"\x04\x16"     , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_diff_chinese_korean     ,     "chinese"    ,         "korean" , b"\xA7\xA8"     , b"\xAC\xA8"); // Ж
    test_trans_simple!(trans_diff_korean_big5_hkscs  ,      "korean"    ,     "big5-hkscs" , b"\xAC\xA8"     , b"\xC7\xFA"); // Ж
    test_trans_simple!(trans_diff_big5_hkscs_gbk     ,  "big5-hkscs"    ,            "gbk" , b"\xC7\xFA"     , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_diff_gbk_sjis           ,         "gbk"    ,           "sjis" , b"\xA7\xA8"     , b"\x84\x47"); // Ж
    test_trans_simple!(trans_diff_sjis_eucjp         ,        "sjis"    ,         "euc-jp" , b"\x84\x47"     , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_diff_eucjp_latin1       ,       "euc-jp"   ,         "latin1" , b"\x8F\xA2\xED" , b"\xA9"    ); // ©
    test_trans_simple!(trans_diff_latin1_utf8        ,      "latin1"    ,          "utf-8" , b"\xA9"         , b"\xC2\xA9"); // ©
}
