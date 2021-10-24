use crate::option;
use crate::transcode;
use crate::error;

use encoding_rs as enc;
use transcoding_rs as tc;
use std::io;
use std::fs;
use std::path;

pub fn dispatch(opt: &option::Opt) -> Result<(), error::Error> {
    if opt.list {
        list();
        return Ok(());
    } else if opt.version {
        version();
        return Ok(());
    } else {
        return run(opt);
    }
}

fn run(opt: &option::Opt) -> Result<(), error::Error> {

    let to_code = match enc::Encoding::for_label(opt.to_code.as_bytes()) {
        None => return Err(error::Error::other(&format!("Invalid encoding: {}", opt.to_code))),
        Some(e) => e,
    };

    let stdout = std::io::stdout();
    let mut stdout_lock;
    let mut writer: Option<&mut dyn io::Write>=None;
    let mut dir_opt: Option<&path::PathBuf>=None;
    let in_paths = &opt.paths;
    match opt.output.as_ref() {
        Some(out_path) => {
            if in_paths.len() == 0 {
                stdout_lock = stdout.lock();
                writer = Some(&mut stdout_lock);
            } else {
                if ! out_path.is_dir() {
                    fs::create_dir(&out_path)
                        .map_err(|e| error::Error::Io { source: e, path: out_path.to_owned(), message: "Error creating the directory".into() })?;
                }
                dir_opt = Some(out_path);
            }
        },
        None => {
            stdout_lock = stdout.lock();
            writer = Some(&mut stdout_lock);
        },
    };

    if in_paths.len() == 0 {
        let stdin = &mut std::io::stdin();
        let rslt = transcode::transcode(stdin, writer.unwrap(), to_code, &opt);
        match rslt {
            Ok(enc) => {
                if opt.show {
                    println!("-: {}", enc.name());
                }
                return Ok(());
            },
            Err(err) => {
                if let error::TranscodeError::Guess(msg) = &err {
                    if opt.quiet {
                        return Ok(());
                    }
                    eprintln!("-: {}", msg);
                }
                return Err(err.into());
            }
        }
    } else {
        for i in 0..in_paths.len() {
            let in_path = &in_paths[i];
            if ! path::Path::exists(&in_paths[i]) {
                let source = io::Error::new(io::ErrorKind::NotFound, "No such file or directory");
                return Err(error::Error::Io { source, path: in_path.to_owned(), message: "Error opening the file".into() });
            }
        }
        for i in 0..in_paths.len() {
            let in_path = &in_paths[i];
            let in_path_can = &fs::canonicalize(&in_path)
                .map_err(|e| error::Error::Io { source: e, path: in_path.to_owned(), message: "Error reading the file or directory".into() })?;
            traverse(&mut writer, to_code, in_path, dir_opt, in_path, in_path_can, opt)?;
        }
        return Ok(());
    }
}

fn traverse(writer_opt: &mut Option<&mut dyn io::Write>, to_code: &'static enc::Encoding,
     in_path: &path::PathBuf, dir_opt: Option<&path::PathBuf>, in_root: &path::PathBuf, in_root_can: &path::PathBuf, opt: &option::Opt)
    -> Result<(), error::Error> {
    if in_path.is_dir() {
        let next_out_dir_opt= {
            if let Some(current_out_dir) = dir_opt {
                let next_out_dir = current_out_dir.join(in_path.file_name().unwrap());
                if ! next_out_dir.is_dir() {
                    fs::create_dir(&next_out_dir)
                        .map_err(|e| error::Error::Io { source: e, path: next_out_dir.to_owned(), message: "Error creating the directory".into() })?;
                }
                Some(next_out_dir)
            } else {
                None
            }
        };
        let mut result: Result<(), error::Error> = Ok(());
        let dir_ent = fs::read_dir(in_path)
            .map_err(|e| error::Error::Io { source: e, path: in_path.to_owned(), message: "Error reading the directory".into() })?;
        for child in dir_ent {
            let c = child
                .map_err(|e| error::Error::Io { source: e, path: in_path.to_owned(), message: "Error reading the directory".into() })?;
            let child_path = &c.path();
            let ret = traverse(writer_opt, to_code, child_path, next_out_dir_opt.as_ref(), in_root, in_root_can, opt);
            if let Err(err) = ret {
                if err.is_guess() {
                    result = Err(err);
                } else {
                    return Err(err);
                }
            }
        }
        return result;
    } else {
        let mut ofile;
        let writer: &mut dyn io::Write = if let Some(dir_path) = dir_opt {
            let out_path = &dir_path.join(in_path.file_name().unwrap());
            ofile =fs::File::create(out_path)
                .map_err(|e| error::Error::Io { source: e, path: out_path.to_owned(), message: "Error creating the file".into() })?;
            &mut ofile
        } else {
            writer_opt.as_mut().unwrap()
        };
        let reader = &mut fs::File::open(in_path)
            .map_err(|e| error::Error::Io { source: e, path: in_path.to_owned(), message: "Error opening the file".into() })?;
        let rslt = transcode::transcode(reader, writer, to_code, &opt);
        let relative_path = || {
            if let Ok(p) = in_path.strip_prefix(in_root_can) {
                in_root.join(p)
            } else {
                in_path.into()
            }
        };
        match rslt {
            Ok(enc) => {
                if opt.show {
                    println!("{}: {}", relative_path().to_string_lossy(), enc.name());
                }
                return Ok(());
            },
            Err(err) => {
                if let error::TranscodeError::Read(source) = err {
                    return Err(error::Error::Io { source, path: in_path.to_owned(), message: "Error reading the file".into() });
                }
                if let error::TranscodeError::Write(source) = err {
                    return Err(error::Error::Io { source, path: in_path.to_owned(), message: "Error writing the file".into() });
                }
                if let error::TranscodeError::Guess(msg) = &err {
                    if opt.quiet {
                        return Ok(());
                    }
                    eprintln!("{}: {}", relative_path().to_string_lossy(), msg);
                }
                return Err(err.into());
            }
        }
    }
}

fn list() {
    print!("{}", tc::ENCODINGS[0].1);
    for i in 1..tc::ENCODINGS.len() {
        let encoding = tc::ENCODINGS[i];
        if tc::ENCODINGS[i-1].0 == encoding.0 {
            print!(" ");
        } else {
            println!();
        }
        print!("{}", encoding.1);
    }
    println!();
}

fn version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

