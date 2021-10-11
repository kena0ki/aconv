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
    pub fn transcode(self: &mut Self, src: & [u8], dst: & mut [u8], last: bool) -> (usize, usize) {
        let is_encoder_utf8 = self.encoder.encoding() == enc::UTF_8;
        if is_encoder_utf8 {
            let (_, num_decoder_read, num_decoder_written, _) =
                self.decoder.decode_to_utf8(src, dst, last);
            return (num_decoder_read, num_decoder_written);
        } else {
            let (_, num_decoder_read, num_decoder_written, _) =
                self.decoder.decode_to_utf8(src, &mut self.decode_buffer, last);
            self.unencoded_bytes.append(&mut self.decode_buffer[..num_decoder_written].to_vec());
            let encoder_input = unsafe {
                str::from_utf8_unchecked(&self.unencoded_bytes)
            };
            let (_, num_encoder_read, num_encoder_written, _) =
                self.encoder.encode_from_utf8(encoder_input, dst, last);
            self.unencoded_bytes = self.unencoded_bytes[num_encoder_read..].to_vec();
            return (num_decoder_read, num_encoder_written);
        }
    }
}

