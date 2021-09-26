use chardetng as cd;

pub fn guess() {
    // utf16
    let mut hoge = cd::EncodingDetector::new();
    // let string = b"\x72\xac\x30\x68\x73\x2b"; // 犬と猫 in UTF-16
    // let string = b"\xfe\xff\x30\x88\x30\x46\x30\x53\x30\x5d"; // ようこそ in UTF-16
    let string = b"\x30\x88\x30\x46\x30\x53\x30\x5d"; // ようこそ in UTF-16
    hoge.feed(string, false);
    let (fuga, success) = hoge.guess_assess(None, true);
    println!("{:?}, {:?}", success, fuga.decode(string));

    // utf8
    let mut hoge = cd::EncodingDetector::new();
    // let string = "こんにちは".as_bytes(); // こんにちは in UTF-8
    // let string = b"\xe3\x82\x88\xe3\x81\x86\xe3\x81\x93\xe3\x81\x9d"; // ようこそ in UTF-8
    let string = "よ".as_bytes(); // ようこそ in UTF-8
    hoge.feed(string, false);
    let fuga = hoge.guess(None, true);
    println!("{:?}, {:?}", success, fuga.decode(string));

    // sjis
    let string = b"\x82\xe6\x82\xa8\x82\xb1\x82\xbb"; // ようこそ in Shift-JIS
    let mut hoge = cd::EncodingDetector::new();
    hoge.feed(string, false);
    let fuga = hoge.guess(None, true);
    println!("{:?}, {:?}", success, fuga.decode(string));

    // eucjp
    let string = b"\xa4\xe8\xa4\xa6\xa4\xb3\xa4\xbd"; // ようこそ in eucjp
    let mut hoge = cd::EncodingDetector::new();
    hoge.feed(string, false);
    let fuga = hoge.guess(None, true);
    println!("{:?}, {:?}", success, fuga.decode(string));
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
        super::guess();
    }
}

