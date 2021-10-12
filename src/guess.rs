use encoding_rs as enc;
use chardetng as cd;

pub struct GuessResult {
    pub encoding: &'static enc::Encoding,
    pub num_fed: usize,
    pub eof: bool,
}

pub fn guess(src: &mut [u8], eof: bool, num_non_aschii: usize) -> GuessResult {

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
        if det.feed(&[*b], eof && exhausted) {
            aschii_cnt+=1;
            if aschii_cnt > num_non_aschii {
                break;
            }
        }
    }
    let top_level_domain = None;
    let allow_utf8 = true;
    let guessed = det.guess(top_level_domain, allow_utf8);
    return GuessResult {
        encoding: guessed,
        num_fed,
        eof: eof && exhausted,
    };
}


#[cfg(test)]
mod tests {
    use crate::transcoder;
//     #[test]
//     fn guess_test() {
//         // utf16
//         let mut hoge = super::cd::EncodingDetector::new();
//         // let string = b"\x72\xac\x30\x68\x73\x2b"; // 犬と猫 in UTF-16
//         // let string = b"\xfe\xff\x30\x88\x30\x46\x30\x53\x30\x5d"; // ようこそ in UTF-16
//         let string = b"\xfe\xff\x30\x88\x30\x46\x30\x53\x30\x5d\x00\x0a"; // ようこそ in UTF-16 without BOM
//         hoge.feed(string, false);
//         let (fuga, success) = hoge.guess_assess(None, true);
//         println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));
// 
//         // utf16 BOMless
//         let mut hoge = super::cd::EncodingDetector::new();
//         let string = b"\x30\x88\x30\x46\x30\x53\x30\x5d"; // ようこそ in UTF-16 without BOM
//         hoge.feed(string, false);
//         let (fuga, success) = hoge.guess_assess(None, true);
//         println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));
// 
//         // utf8
//         let mut hoge = super::cd::EncodingDetector::new();
//         // let string = "こんにちは".as_bytes(); // こんにちは in UTF-8
//         // let string = b"\xe3\x82\x88\xe3\x81\x86\xe3\x81\x93\xe3\x81\x9d"; // ようこそ in UTF-8
//         let string = "よ".as_bytes(); // ようこそ in UTF-8
//         hoge.feed(string, false);
//         let (fuga, success) = hoge.guess_assess(None, true);
//         println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));
// 
//         // sjis
//         let string = b"\x82\xe6\x82\xa8\x82\xb1\x82\xbb"; // ようこそ in Shift-JIS
//         let mut hoge = super::cd::EncodingDetector::new();
//         hoge.feed(string, false);
//         let (fuga, success) = hoge.guess_assess(None, true);
//         println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));
// 
//         // eucjp
//         let string = b"\xa4\xe8\xa4\xa6\xa4\xb3\xa4\xbd"; // ようこそ in eucjp
//         let mut hoge = super::cd::EncodingDetector::new();
//         hoge.feed(string, false);
//         let (fuga, success) = hoge.guess_assess(None, true);
//         println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));
// 
//         // binary data
//         let string = b"\x00\xe8\xa4\x00\xa4\x00\xa4\x00"; // random bynary data
//         let mut hoge = super::cd::EncodingDetector::new();
//         hoge.feed(string, false);
//         let (fuga, success) = hoge.guess_assess(None, true);
//         println!("{:?}, {:?}, {:?}", success, fuga, fuga.decode(string));
//     }
}

