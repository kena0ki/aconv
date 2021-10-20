use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs
use walkdir::WalkDir;
use std::io::Read;
use insta;

#[test]
fn dir_to_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let cmd = cmd.arg("test_data/dir_to_dir")
        .args(&["-o","output"])
        .current_dir(std::path::PathBuf::from(".").canonicalize()?);
    cmd.assert().success();
    assert_directories("test_data/dir_to_dir", "output/dir_to_dir");
    Ok(())
}

#[test]
fn files_to_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let cmd = cmd.arg("test_data/files_to_dir/file1.txt")
        .arg("test_data/files_to_dir/file2.txt")
        .args(&["-o","output/files_to_dir"])
        .current_dir(std::path::PathBuf::from(".").canonicalize()?);
    cmd.assert().success();
    assert_directories("test_data/files_to_dir", "output/files_to_dir");
    Ok(())
}

#[test]
fn to_code() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let cmd = cmd.arg("test_data/to_code/eucjp_to_sjis.txt")
        .args(&["-t","sjis"])
        .args(&["-o","output/to_code"])
        .current_dir(std::path::PathBuf::from(".").canonicalize()?);
    cmd.assert().success();

    let mut f = std::fs::File::open("output/to_code/eucjp_to_sjis.txt").unwrap();
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).unwrap();
    assert_eq!(b"\x83\x6E\x83\x8D\x81\x5B\x83\x8F\x81\x5B\x83\x8B\x83\x68\x0A", &(*bytes));

    Ok(())
}
//
//    #[test]
//    fn list() {
//        let opt = &mut option::Opt::new()
//            .list(true);
//        dispatch(opt).unwrap();
//    }
//
//    #[test]
//    fn show() {
//        let opt = &mut option::Opt::new()
//            .paths(vec![path::PathBuf::from("test_data/files_to_dir")])
//            .show(true);
//        dispatch(opt).unwrap();
//    }

#[test]
fn list() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let output = cmd.arg("-v").output().unwrap();
    insta::assert_debug_snapshot!(output.stdout);
    Ok(())
}

#[test]
fn show() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let output = cmd.arg("test_data/to_code").arg("-s").unwrap();
    insta::assert_debug_snapshot!(output.stdout);
    Ok(())
}

#[test]
fn version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    cmd.arg("-v");
    cmd.assert()
        .success()
    //    .stdout(predicate::str::contains(format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))));
        .stdout("");
    Ok(())
}

#[test]
fn error_noent() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;

    cmd.arg("foobar").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}

fn assert_directories(left_str: &str, right_str: &str) {
    let left_root = std::path::PathBuf::from(left_str);
    let right_root = std::path::PathBuf::from(right_str);
    if !left_root.is_dir() {
        panic!("{:?} is not a directory", left_root);
    } else if !right_root.is_dir() {
        panic!("{:?} is not a directory", right_root);
    }
    let mut left_dirs: Vec<_> = WalkDir::new(left_str).into_iter().filter_map(|e| e.ok()).map(|x| x.into_path()).collect();
    left_dirs.sort_by(|a,b| a.cmp(&b));
    let mut right_dirs: Vec<_> = WalkDir::new(right_str).into_iter().filter_map(|e| e.ok()).map(|x| x.into_path()).collect();
    right_dirs.sort_by(|a,b| a.cmp(&b));
    if left_dirs.len() != right_dirs.len() {
        panic!("Directories does not have same contents. \nleft {:?}\nright: {:?}", left_dirs, right_dirs);
    }
    for i in 0..left_dirs.len() {
        let left = &left_dirs[i];
        let right = &right_dirs[i];
        if left.strip_prefix(&left_root).unwrap() != right.strip_prefix(&right_root).unwrap() {
            panic!("Directories does not have same contents. \nleft {:?}\nright: {:?}", left_dirs, right_dirs);
        }
    }
    for i in 0..left_dirs.len() {
        let left = &left_dirs[i];
        let right = &right_dirs[i];
        let left_buf = &mut Vec::with_capacity(1024);
        let right_buf = &mut Vec::with_capacity(1024);
        if !left.is_dir() {
            let mut l = std::fs::File::open(left).unwrap();
            let mut r = std::fs::File::open(right).unwrap();
            l.read_to_end (left_buf).unwrap();
            r.read_to_end (right_buf).unwrap();
            if left_buf != right_buf {
                panic!("Files are not equal. left {:?}, right: {:?}", left, right);
            }
        }
    }
}
//
//    #[test]
//    fn threshold() {
//        let opt = &mut option::Opt::new()
//            .paths(vec![path::PathBuf::from("test_data/threshold/threshold.txt")])
//            .non_text_threshold(50);
//        dispatch(opt).unwrap();
//    }
//
//    #[test]
//    fn error_guess() {
//        let opt = &mut option::Opt::new()
//            .paths(vec![path::PathBuf::from("test_data/error_guess/nullchars.txt")]);
//        dispatch(opt).unwrap_err();
//    }
//
//    #[test]
//    fn error_guess_quiet() {
//        let opt = &mut option::Opt::new()
//            .paths(vec![path::PathBuf::from("test_data/error_guess/nullchars.txt")])
//            .quiet(true);
//        dispatch(opt).unwrap_err();
//    }
