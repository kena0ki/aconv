use encoding_rs as enc;
use chardetng as cd;

pub struct GuessResult {
    pub encoding: &'static enc::Encoding,
    pub num_fed: usize,
    pub exhausted: bool,
}

pub fn guess(src: &mut [u8], num_non_aschii: usize) -> GuessResult {
    let num_read = src.len();
    let mut det = cd::EncodingDetector::new();
    let mut aschii_cnt = 0;
    let mut num_fed = 0;
    let mut exhausted = num_read == num_fed;
    for b in src.iter() {
        num_fed+=1;
        exhausted = num_read == num_fed;
        if det.feed(&[*b], exhausted) {
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
        exhausted,
    };
}
