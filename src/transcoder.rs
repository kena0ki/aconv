use encoding_rs as enc;
use std::str;

pub struct Transcoder<'a> {
    decoder: &'a mut enc::Decoder,
    encoder: &'a mut enc::Encoder,
    decode_buffer: Vec<u8>,
    unencoded_bytes: Vec<u8>,
}

impl<'a> Transcoder<'a> {
    pub fn new(decoder: &'a mut enc::Decoder, encoder: &'a mut enc::Encoder) -> Self {
        return Transcoder {
            decoder,
            encoder,
            decode_buffer: vec![0u8;8 * 1024],
            unencoded_bytes: Vec::with_capacity(8 * 1024),
        };
    }
    pub fn new_with_buff_size(decoder: &'a mut enc::Decoder, encoder: &'a mut enc::Encoder, size: usize) -> Result<Self, String> {
        if size < 4 { // at least 3 bytes are required for encoding_rs to write a valid UTF-8 character.
            let msg = format!("Buffer of size {} is too small", size);
            return Err(msg);
        }
        return Ok(Transcoder {
            decoder,
            encoder,
            decode_buffer: vec![0u8;size],
            unencoded_bytes: Vec::with_capacity(size),
        });
    }
    pub fn transcode(self: &mut Self, src: & [u8], dst: & mut [u8], last: bool) -> (enc::CoderResult, usize, usize) {
        let is_encoder_utf8 = self.encoder.encoding() == enc::UTF_8;
        if is_encoder_utf8 {
            let (result, num_decoder_read, num_decoder_written, _) =
                self.decoder.decode_to_utf8(src, dst, last);
            return (result, num_decoder_read, num_decoder_written);
        } else {
            let (decoder_result, num_decoder_read, num_decoder_written, _) =
                self.decoder.decode_to_utf8(src, &mut self.decode_buffer, last);
            self.unencoded_bytes.append(&mut self.decode_buffer[..num_decoder_written].to_vec());
            let encoder_input = unsafe {
                str::from_utf8_unchecked(&self.unencoded_bytes)
            };
            let (encoder_result, num_encoder_read, num_encoder_written, _) =
                self.encoder.encode_from_utf8(encoder_input, dst, last);
            self.unencoded_bytes = self.unencoded_bytes[num_encoder_read..].to_vec();
            let result = if decoder_result == enc::CoderResult::InputEmpty && encoder_result == enc::CoderResult::InputEmpty {
                enc::CoderResult::InputEmpty
            } else {
                enc::CoderResult::OutputFull
            };
            return (result, num_decoder_read, num_encoder_written);
        }
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
                assert_eq!($dst, &output[..written]);
            }
        };
    }

    // This isn't exhaustive obviously, but it lets us test base level support.
    test_trans_simple!(trans_simple_utf8       ,        "utf-8"   ,          "utf-8" , b"\xD0\x96" , b"\xD0\x96"); // Ж
    test_trans_simple!(trans_simple_utf16le    ,     "utf-16le"   ,       "utf-16le" , b"\x16\x04" , b"\x16\x04"); // Ж
    test_trans_simple!(trans_simple_utf16be    ,     "utf-16be"   ,       "utf-16be" , b"\x04\x16" , b"\x04\x16"); // Ж
    test_trans_simple!(trans_simple_chinese    ,     "chinese"    ,        "chinese" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_simple_korean     ,      "korean"    ,         "korean" , b"\xAC\xA8" , b"\xAC\xA8"); // Ж
    test_trans_simple!(trans_simple_big5_hkscs ,  "big5-hkscs"    ,     "big5-hkscs" , b"\xC7\xFA" , b"\xC7\xFA"); // Ж
    test_trans_simple!(trans_simple_gbk        ,         "gbk"    ,            "gbk" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_simple_sjis       ,        "sjis"    ,           "sjis" , b"\x84\x47" , b"\x84\x47"); // Ж
    test_trans_simple!(trans_simple_eucjp      ,       "euc-jp"   ,         "euc-jp" , b"\xA7\xA8" , b"\xA7\xA8"); // Ж
    test_trans_simple!(trans_simple_latin1     ,      "latin1"    ,         "latin1" , b"\xA9"     , b"\xA9"    ); // ©
}
