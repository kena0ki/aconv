use encoding_rs as enc;
use chardetng as cd;

pub struct GuessResult {
    pub encoding: &'static enc::Encoding,
    pub num_fed: usize,
    pub eof: bool,
}

pub fn guess(src: &mut [u8], eof: bool) -> GuessResult {
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
        num_fed,
        eof: eof && exhausted,
    };
}
