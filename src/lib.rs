mod option;

use chardetng as cd;
use std::io::Read;
use std::fs;

pub use option::Opt;

pub fn guess() {
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

pub fn guess_file(opt: option::Opt) {
    if let Some(path) = opt.files.iter().next() {
        let mut buf = vec![];
        let mut file = fs::File::open(path).unwrap();
        file.read_to_end(&mut buf);
    }
}


#[cfg(test)]
mod tests {
    use std::io::Read;

    #[test]
    fn it_works() {
        let s = "hoge";
        let t = s;
        println!("{}, {}",s, t);
        assert_eq!(2 + 2, 4);
        // println!("{}",1 << 10);
        // println!("{}",2_i32.pow(10));
        super::guess();
    }
    #[test]
    fn gif() {
        let opt = super::Opt::new();
        let mut buf = vec![];
        let mut file = super::fs::File::open("demo.gif").unwrap();
        file.read_to_end(&mut buf);
        super::guess_file();
    }
}

