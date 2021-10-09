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
    controller(opt, &mut file, &mut stdout)
}

pub fn controller(opt: option::Opt, read: &mut dyn io::Read, write: &mut io::Write) {

    // BOM sniffing

    // guess
    let mut buf = vec![0; 5 * (1 << 10)]; // first 5K bytes
    let mut buf_next = vec![0; 1]; // variable to check if eof
    let buf_size=read.read(&mut buf).unwrap_or(0);
    let buf_next_size=read.read(&mut buf_next).unwrap_or(0);
    let eof = buf_next_size == 0;
    let mut guess_ok = guess(&buf[..buf_size], eof);

    // try_decode_first
    let (mut decoder, decoded_string, last, auto_detection_failed)
        = try_decode_first_bytes(&mut guess_ok, &mut buf);
    if auto_detection_failed {
        if let Err(cause) = write.write_all(&buf) {
            exit_with_io_error("Error writing output", cause);
        }
        if let Err(cause) = io::copy(read, write) {
            exit_with_io_error("Error writing output", cause);
        }
        return;
    }
    if last {
        if let Err(cause) = write.write_all(decoded_string.as_bytes()) {
            exit_with_io_error("Error writing output", cause);
        }
        return;
    }

    // decode
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

fn guess(buf: &[u8], eof: bool) -> GuessResult {
    let buf_size = buf.len();
    let mut det = cd::EncodingDetector::new();
    let mut aschii_cnt = 0;
    let mut byte_cnt = 0;
    for b in buf.iter() {
        byte_cnt+=1;
        let exhausted = buf_size == byte_cnt;
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
        num_read: buf_size,
        num_fed: byte_cnt,
        eof,
    };
}

fn try_decode_first_bytes(guess_file_ok: &mut GuessResult, buf: &[u8]) -> (enc::Decoder, String, bool, bool) {
    let GuessResult{ encoding, num_read, num_fed, eof } = *guess_file_ok;
    let mut decoder = encoding.new_decoder();
    let last = eof && num_read == num_fed;
    let decoded_string = {
        let mut decoded_string = String::new();
        let mut decoder_input_start = 0;
        loop {
            let (decoder_result, decoder_read, _) =
                decoder.decode_to_string(&buf[decoder_input_start..num_fed], &mut decoded_string, last);
            decoder_input_start += decoder_read;
            if decoder_result == enc::CoderResult::InputEmpty {
                break;
            }
        }
        decoded_string
    };
    let mut non_text_cnt = 0;
    for s in decoded_string.chars() {
        if let Ok(_) = constants::NON_TEXTS_FREQUENT.binary_search(&s) {
            non_text_cnt+=1;
            continue;
        }
        if let Ok(_) = constants::NON_TEXTS.binary_search(&s) {
            non_text_cnt+=1;
        }
    }
    let auto_detection_failed = 0 < (num_fed / non_text_cnt);
    (decoder, decoded_string, last, auto_detection_failed)
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
