use encoding_rs as enc;

pub struct Transcoder<'a> {
    decoder: &'a mut enc::Decoder,
    encoder: &'a mut enc::Encoder,
    decode_buffer_str: &'a mut str,
    unencoded_string: String,
}

impl<'a> Transcoder<'a> {
    pub fn new(decoder: &'a mut enc::Decoder, encoder: &'a mut enc::Encoder, buffer: &'a mut[u8]) -> Self {
        let decode_buffer_str = unsafe {
            // str::from_utf8_mut can cast &[u8] to &str but there is no reason for buffer being validated to be
            // utf8 since buffer is used as a byte array in Decoder::decode_to_str.
            std::mem::transmute(&mut buffer[..])
        };
        return Transcoder {
            decoder,
            encoder,
            decode_buffer_str,
            unencoded_string: String::new(),
        };
    }
    pub fn transcode(self: &mut Self, src: & [u8], dst: & mut [u8], last: bool) -> (usize, usize) {
        let is_encoder_utf8 = self.encoder.encoding() == enc::UTF_8;
        if is_encoder_utf8 {
            let (_, num_decoder_read, num_decoder_written, _) =
                self.decoder.decode_to_utf8(src, dst, last);
            return (num_decoder_read, num_decoder_written);
        } else {
            let (_, num_decoder_read, num_decoder_written, _) =
                self.decoder.decode_to_str(src, self.decode_buffer_str, last);
            self.unencoded_string.push_str((&self.decode_buffer_str[..num_decoder_written]).into());
            let (_, num_encoder_read, num_encoder_written, _) =
                self.encoder.encode_from_utf8(self.unencoded_string.as_str(), dst, last);
            self.unencoded_string = (&self.decode_buffer_str[num_encoder_read..num_decoder_written]).into();
            return (num_decoder_read, num_encoder_written);
        }
    }
}

