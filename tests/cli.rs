// use assert_cmd::prelude::*; // Add methods on commands
use assert_cmd::Command; // Run programs
use walkdir::WalkDir;
use std::io::Read;
use insta;

#[test]
fn dir_to_dir() -> Result<(), Box<dyn std::error::Error>> {
    let empty_dir = std::path::PathBuf::from("test_data/dir_to_dir/child/empty_dir/");
    if ! empty_dir.is_dir() {
        std::fs::create_dir(empty_dir).unwrap(); // Git can't track empty dirs, so let's make it.
    }
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

#[test]
fn threshold() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let output = cmd.args(&["-T","50"])
        .write_stdin("a\x00b\x00c\x00d\x00e\x00")
        .unwrap();
    insta::assert_debug_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
    Ok(())
}

#[test]
fn list() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let output = cmd.arg("-l").unwrap();
    insta::assert_display_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
    Ok(())
}

#[test]
fn show() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let output = cmd.arg("test_data/to_code").arg("-s").unwrap();
    insta::assert_display_snapshot!(std::str::from_utf8(&output.stdout).unwrap());
    Ok(())
}

#[test]
fn version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let assert = cmd.arg("-v").assert();
    let expected = format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    assert.stdout(expected);
    Ok(())
}

#[test]
fn error_noent() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let assert = cmd.arg("test/file/doesnt/exist").assert().failure();
    let output = assert.get_output();
    insta::assert_debug_snapshot!(std::str::from_utf8(&output.stderr).unwrap());
    Ok(())
}

#[test]
fn error_guess() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let assert = cmd.write_stdin("a\x00b\x00c\x00d\x00e\x00").assert().failure();
    let output = assert.get_output();
    insta::assert_debug_snapshot!("error_guess_stdout", std::str::from_utf8(&output.stdout).unwrap());
    insta::assert_debug_snapshot!("error_guess_stderr", std::str::from_utf8(&output.stderr).unwrap());
    Ok(())
}

#[test]
fn error_guess_quiet() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let assert = cmd.arg("-q").write_stdin("a\x00b\x00c\x00d\x00e\x00").assert().success();
    let output = assert.get_output();
    insta::assert_debug_snapshot!("error_guess_quiet_stdout", std::str::from_utf8(&output.stdout).unwrap());
    insta::assert_debug_snapshot!("error_guess_quiet_stderr", std::str::from_utf8(&output.stderr).unwrap());
    Ok(())
}

#[test]
fn error_guess_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let assert = cmd.arg("test_data/threshold/threshold.txt").assert().failure();
    let output = assert.get_output();
    insta::assert_debug_snapshot!("error_guess_file_stdout", std::str::from_utf8(&output.stdout).unwrap());
    insta::assert_debug_snapshot!("error_guess_file_stderr", std::str::from_utf8(&output.stderr).unwrap());
    Ok(())
}

#[test]
fn error_guess_file_quiet() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let assert = cmd.arg("test_data/threshold/threshold.txt")
        .arg("-q").write_stdin("a\x00b\x00c\x00d\x00e\x00").assert().success();
    let output = assert.get_output();
    insta::assert_debug_snapshot!("error_guess_file_quiet_stdout", std::str::from_utf8(&output.stdout).unwrap());
    insta::assert_debug_snapshot!("error_guess_file_quiet_stderr", std::str::from_utf8(&output.stderr).unwrap());
    Ok(())
}

#[test]
fn error_guess_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aconv")?;
    let assert = cmd.arg("test_data/error_guess_dir").assert().failure();
    let output = assert.get_output();
    insta::assert_debug_snapshot!("error_guess_dir_stdout", std::str::from_utf8(&output.stdout).unwrap());
    insta::assert_display_snapshot!("error_guess_dir_stderr", std::str::from_utf8(&output.stderr).unwrap());
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
