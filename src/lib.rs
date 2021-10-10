mod option;
mod constants;

use chardetng as cd;
use encoding_rs as enc;
use std::io;
use std::fs;
use std::process;

pub use option::Opt;

pub fn cli(opt: option::Opt) {
    // check
    let mut file = match opt.paths.iter().next() {
        Some(path) => fs::File::open(path).unwrap(),
        None => return,
    };

    let mut stdout = std::io::stdout();
    let to_code = enc::Encoding::for_label(opt.to.as_bytes()).unwrap_or(&enc::UTF_8_INIT);
    let mut encoder = to_code.new_encoder();
    controller(&mut file, &mut stdout, &mut encoder);
}

pub fn controller(read: &mut impl io::Read, write: &mut impl io::Write, encoder: &mut enc::Encoder) {

    let mut input_buffer = [0u8; 5 * (1 << 10)]; // 5K bytes
    let mut decode_buffer = [0u8; 10 * (1 << 10)]; // 10K bytes
    let mut decode_buffer_str =
        std::str::from_utf8_mut(&mut decode_buffer[..]).unwrap();

    // BOM sniffing

    // guess the input encoding using up to a few Kbytes of byte sequences
    let mut guess_ok = guess(read, &mut input_buffer);

    // try to decode byte sequences being used to guess
    let mut buf_first_read = &input_buffer[..guess_ok.num_read];
    let (mut decoder, decoder_read, decoder_written, auto_detection_failed)
        = try_decode_first_bytes(&mut guess_ok, &mut buf_first_read, &mut decode_buffer_str);
    if auto_detection_failed {
        try_write(|| write.write_all(&buf_first_read));
        try_write(|| io::copy(read, write).map(|_| ()));
        return;
    }
    try_write(|| write.write_all(&decode_buffer_str.as_bytes()[..decoder_written]));
    if guess_ok.eof && decoder_read >= guess_ok.num_fed {
        return;
    }

    // decode rest of bytes in buffer

    // decode bytes remaining in file
    decode_file(read, write, &mut decoder, &mut input_buffer, decode_buffer_str, decoder_read, encoder);
}

fn decode_file(read: &mut impl io::Read,write: &mut impl io::Write, decoder: &mut enc::Decoder,
    input_buffer: &mut [u8], decode_buffer_str: &mut str, mut decoder_input_start: usize, encoder: &mut enc::Encoder) {
    let mut first = true;
    loop {
        match read.read(input_buffer) {
            Ok(n) => {
                let eof = n == 0;
                if ! first {
                    decoder_input_start = 0;
                } else {
                    first = false;
                }
                conv(write, decoder, input_buffer, decode_buffer_str, decoder_input_start, n, eof, encoder);
                if eof {
                    break;
                }
            }
            Err(cause) => {
                exit_with_io_error("Error reading input", cause);
            }
        }
    };
}

struct IoParam {
    write: io::Write,
}

fn conv(write: &mut impl io::Write, decoder: &mut enc::Decoder, input_buffer: &mut [u8], decode_buffer_str: &mut str,
    mut decoder_input_start: usize, decoder_input_end: usize, eof: bool, encoder: &mut enc::Encoder) {
    loop {
        let (decoder_result, decoder_read, decoder_written, _) =
            decoder.decode_to_str(&input_buffer[decoder_input_start..decoder_input_end], decode_buffer_str, eof);
        decoder_input_start += decoder_read;
        if decoder_result == enc::CoderResult::InputEmpty {
            break;
        }
        if encoder.encoding() == enc::UTF_8 {
            try_write(|| write.write_all(&decode_buffer_str.as_bytes()[..decoder_written]));
        } else {
            let mut encoder_input_start = 0;
            loop {
                let (encoder_result, encoder_read, encoder_written, _) =
                    encoder.encode_from_utf8(&decode_buffer_str[encoder_input_start..decoder_written], input_buffer, eof);
            }
        }
    }
}


fn try_write(mut fnc: impl FnMut() -> Result<(),io::Error>) {
    if let Err(cause) = fnc() {
        exit_with_io_error("Error writing output", cause);
    }
}

fn exit_with_io_error(message: &str, cause: io::Error) {
    eprintln!("{}: {}", message, cause);
    process::exit(constants::IO_ERROR);
}

struct GuessResult {
    encoding: &'static enc::Encoding,
    num_read: usize,
    num_fed: usize,
    eof: bool,
}

fn guess(read: &mut impl io::Read, input_buffer: &mut [u8]) -> GuessResult {
    let (buf_first_read, eof) = {
        let mut buf_next = [0; 1]; // buffer to check if eof
        let buf_size=read.read(input_buffer).unwrap_or(0);
        let buf_next_size=read.read(&mut buf_next).unwrap_or(0);
        let buf_first_read = [&input_buffer[..buf_size], &buf_next[..buf_next_size]].concat();
        let eof = buf_next_size == 0;
        (buf_first_read, eof)
    };
    let num_read = buf_first_read.len();
    let mut det = cd::EncodingDetector::new();
    let mut aschii_cnt = 0;
    let mut num_fed = 0;
    let mut exhausted = num_read == num_fed;
    for b in buf_first_read.iter() {
        num_fed+=1;
        exhausted = num_read == num_fed;
        if det.feed(&[*b], eof && exhausted) {
            aschii_cnt+=1;
            if aschii_cnt > 1000 { // above 1000 non-aschii bytes
                break;
            }
        }
    }
    let top_level_domain = None;
    let allow_utf8 = true;
    let guessed = det.guess(top_level_domain, allow_utf8);
    return GuessResult {
        encoding: guessed,
        num_read,
        num_fed,
        eof: eof && exhausted,
    };
}

fn try_decode_first_bytes(guess_file_ok: &mut GuessResult, buf: &[u8], decode_buffer_str: &mut str)
    -> (enc::Decoder, usize, usize, bool) {
    let GuessResult{ encoding, num_fed, num_read:_, eof } = *guess_file_ok;
    let mut decoder = encoding.new_decoder();
    let (_, decoder_read, decoder_written, _) = decoder.decode_to_str(&buf[..num_fed], decode_buffer_str, eof);
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
    let auto_detection_failed = 0 < (decode_buffer_str.chars().count() / non_text_cnt);
    (decoder, decoder_read, decoder_written, auto_detection_failed)
}

fn is_binary() {
}

#[cfg(test)]
mod tests {
    use std::path;

    // #[test]
    // fn it_works() {
    //     let s = "hoge";
    //     let t = s;
    //     println!("{}, {}",s, t);
    //     assert_eq!(2 + 2, 4);
    //     // println!("{}",1 << 10);
    //     // println!("{}",2_i32.pow(10));
    //     super::guess();
    // }
    #[test]
    fn gif() {
        let mut opt = super::Opt::default();
        let path = path::PathBuf::from("src/test_data/demo.gif");
        opt.paths.push(path);
        let (fuga, success) = super::guess(opt).expect("couldn't guess");
        println!("{:?}, {:?}", success, fuga);
        let buf = fuga.decode(buf_slice);
    }
}


fn guess_test() {
    // utf16
    let mut hoge = cd::EncodingDetector::new();
    // let string = b"\x72\xac\x30\x68\x73\x2b"; // 犬と猫 in UTF-16
    // let string = b"\xfe\xff\x30\x88\x30\x46\x30\x53\x30\x5d"; // ようこそ in UTF-16
    let string = b"\xfe\xff\x30\x88\x30\x46\x30\x53\x30\x5d\x00\x0a"; // ようこそ in UTF-16 without BOM
    hoge.feed(string, false);
    let (fuga, success) = hoge.guess_assess(None, true);
    println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));

    // utf16 BOMless
    let mut hoge = cd::EncodingDetector::new();
    let string = b"\x30\x88\x30\x46\x30\x53\x30\x5d"; // ようこそ in UTF-16 without BOM
    hoge.feed(string, false);
    let (fuga, success) = hoge.guess_assess(None, true);
    println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));

    // utf8
    let mut hoge = cd::EncodingDetector::new();
    // let string = "こんにちは".as_bytes(); // こんにちは in UTF-8
    // let string = b"\xe3\x82\x88\xe3\x81\x86\xe3\x81\x93\xe3\x81\x9d"; // ようこそ in UTF-8
    let string = "よ".as_bytes(); // ようこそ in UTF-8
    hoge.feed(string, false);
    let (fuga, success) = hoge.guess_assess(None, true);
    println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));

    // sjis
    let string = b"\x82\xe6\x82\xa8\x82\xb1\x82\xbb"; // ようこそ in Shift-JIS
    let mut hoge = cd::EncodingDetector::new();
    hoge.feed(string, false);
    let (fuga, success) = hoge.guess_assess(None, true);
    println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));

    // eucjp
    let string = b"\xa4\xe8\xa4\xa6\xa4\xb3\xa4\xbd"; // ようこそ in eucjp
    let mut hoge = cd::EncodingDetector::new();
    hoge.feed(string, false);
    let (fuga, success) = hoge.guess_assess(None, true);
    println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));

    // binary data
    let string = b"\x00\xe8\xa4\x00\xa4\x00\xa4\x00"; // random bynary data
    let mut hoge = cd::EncodingDetector::new();
    hoge.feed(string, false);
    let (fuga, success) = hoge.guess_assess(None, true);
    println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));
}
