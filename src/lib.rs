pub mod option;
pub mod constants;
pub mod transcoder;
pub mod error;

use encoding_rs as enc;
use std::io;
use std::fs;
use std::path;

pub fn cli(opt: &option::Opt) -> Result<(), error::Error> {
    let input_buffer = &mut [0u8; 5*1024]; // 5K bytes
    let output_buffer = &mut [0u8; 10*1024]; // 10K bytes

    let to_code = match enc::Encoding::for_label(opt.to_code.as_bytes()) {
        None => return Err(error::Error::other(&format!("Invalid encoding: {}", opt.to_code))),
        Some(e) => e,
    };

    let mut ofile;
    let stdout;
    let mut stdout_lock;
    let mut writer: Option<&mut dyn io::Write>=None;
    let mut dir_name: Option<&path::PathBuf>=None;
    match opt.output.as_ref() {
        Some(path) => {
            ofile = fs::File::create(path)
                .map_err(|e| error::Error::Io { source: e, path: path.to_owned(), message: "Error creating the file".into() })?;
            let meta = ofile.metadata()
                .map_err(|e| error::Error::Io { source: e, path: path.to_owned(), message: "Error reading the metadata of the file".into() })?;
            if meta.is_file() {
                writer = Some(&mut ofile)
            } else {
                dir_name = Some(path);
            }
        },
        None => {
            stdout = std::io::stdout();
            stdout_lock = stdout.lock();
            writer = Some(&mut stdout_lock)
        },
    };

    return run(writer, to_code, input_buffer, output_buffer, dir_name ,opt);
}

fn run(writer_opt: Option<&mut dyn io::Write>, to_code: &'static enc::Encoding, input_buffer: &mut [u8], output_buffer: &mut [u8]
    ,dir_name: Option<&path::PathBuf>, opt: &option::Opt)
    -> Result<(), error::Error> {
    let mut ofile;
    let writer: &mut dyn io::Write = if let Some(path) = dir_name {
        ofile =fs::File::create(path)
            .map_err(|e| error::Error::Io { source: e, path: path.to_owned(), message: "Error creating the file".into() })?;
        &mut ofile
    } else {
        writer_opt.unwrap()
    };
    let mut result: Result<(),error::Error> = Ok(());
    let ipaths = &opt.paths;
    if ipaths.len() == 0 {
        let stdin = &mut std::io::stdin();
        let rslt = transcode(stdin, writer, to_code, input_buffer, output_buffer, &opt);
        if let Err(err) = rslt {
            if let error::TranscodeError::Guess(msg) = &err {
                if ! opt.quiet {
                    eprintln!("{}", msg);
                }
            }
            result = Err(err.into());
        }
    } else {
        for i in 0..ipaths.len() {
            let path = &ipaths[i];
            if ! path::Path::exists(&ipaths[i]) {
                let source = io::Error::new(io::ErrorKind::NotFound, "No such file or directory");
                return Err(error::Error::Io { source, path: path.to_owned(), message: "Error opening the file".into() });
            }
        }
        for i in 0..ipaths.len() {
            let path = &ipaths[i];
            let reader = &mut fs::File::open(path)
                .map_err(|e| error::Error::Io { source: e, path: path.to_owned(), message: "Error opning the file".into() })?;
            let rslt = transcode(reader, writer, to_code, input_buffer, output_buffer, &opt);
            if let Err(err) = rslt {
                if let error::TranscodeError::Read(source) = err {
                    return Err(error::Error::Io { source, path: path.to_owned(), message: "Error reading output".into() });
                }
                if let error::TranscodeError::Write(source) = err {
                    return Err(error::Error::Io { source, path: path.to_owned(), message: "Error writing output".into() });
                }
                if let error::TranscodeError::Guess(msg) = &err {
                    if ! opt.quiet {
                        eprintln!("{}", msg);
                    }
                }
                result = Err(err.into());
            }
        };
    }
    return result;
}

pub fn transcode(reader: &mut dyn io::Read, writer: &mut dyn io::Write, encoding: &'static enc::Encoding,
    input_buffer: &mut [u8], output_buffer: &mut [u8], opt: &option::Opt)
    -> Result<&'static enc::Encoding, error::TranscodeError> {

    // guess the input encoding using up to a few Kbytes of byte sequences
    let (mut buf_guess, eof) = {
        let in_size = input_buffer.len()-1; // to check if second read size is 0, -1 from input_buffer size.
        let first_size=reader.read(&mut input_buffer[..in_size]).unwrap_or(0);
        let second_size=reader.read(&mut input_buffer[first_size..]).unwrap_or(0);
        let buf_guess = &mut input_buffer[..(first_size+second_size)];
        let eof = second_size == 0;
        (buf_guess, eof)
    };
    let transcoder = &mut transcoder::Transcoder::new_with_buff_size(None, encoding, 10 * 1024).unwrap();
    let num_read = {
        let rslt = transcoder.guess_and_transcode(&mut buf_guess, output_buffer, opt.chars_to_guess, opt.non_text_threshold, eof);
        match rslt {
            Ok((_, num_read, num_written)) => {
                let should_not_transcode = transcoder.src_encoding().unwrap() == transcoder.dst_encoding();
                if should_not_transcode {
                    // write input to output as-is
                    writer.write_all(&buf_guess).map_err(map_write_err)?;
                    io::copy(reader, writer).map(|_| ()).map_err(map_write_err)?;
                    return Ok(transcoder.src_encoding().unwrap());
                }
                if transcoder.dst_encoding() == enc::UTF_16BE && [0xFE,0xFF] != output_buffer[..2] {
                    writer.write_all(b"\xFE\xFF").map_err(map_write_err)?; // add a BOM
                }
                if transcoder.dst_encoding() == enc::UTF_16LE && [0xFF,0xFE] != output_buffer[..2] {
                    writer.write_all(b"\xFF\xFE").map_err(map_write_err)?; // add a BOM
                }
                // write transcoded bytes in buffer
                writer.write_all(&output_buffer[..num_written]).map_err(map_write_err)?;
                num_read
            },
            Err(err) => {
                // write input to output as-is
                writer.write_all(&buf_guess).map_err(map_write_err)?;
                io::copy(reader, writer).map(|_| ()).map_err(map_write_err)?;
                return Err(error::TranscodeError::Guess(err));
            },
        }
    };

    // decode rest of bytes in buffer
    transcode_buffer_and_write(writer, transcoder, &buf_guess[num_read..], output_buffer, false)?;

    // decode bytes remaining in file
    transcode_file_and_write(reader, writer, transcoder, input_buffer, output_buffer)?;

    return Ok(transcoder.src_encoding().unwrap());
}

fn transcode_file_and_write(reader: &mut dyn io::Read,writer: &mut dyn io::Write, transcoder: &mut transcoder::Transcoder,
    input_buffer: &mut [u8], output_buffer: &mut [u8])
    -> Result<(), error::TranscodeError>{
    loop {
        let num_read = reader.read(input_buffer).map_err(map_read_err)?;
        let eof = num_read == 0;
        transcode_buffer_and_write(writer, transcoder, &input_buffer[..num_read], output_buffer, eof)?;
        if eof {
            break;
        }
    };
    return Ok(());
}

fn transcode_buffer_and_write(writer: &mut dyn io::Write, transcoder: &mut transcoder::Transcoder,
    src: &[u8], output_buffer: &mut [u8], eof: bool) 
    -> Result<(), error::TranscodeError>{
    let mut transcoder_input_start = 0;
    if src.len() == 0 && !eof { // encoding_rs unable to handle unnecessary calls well, so let's skip them
        return Ok(());
    }
    loop {
        let (result, num_transcoder_read, num_transcoder_written)
            = transcoder.transcode(&src[transcoder_input_start..], output_buffer, eof);
        transcoder_input_start+=num_transcoder_read;
        writer.write_all(&output_buffer[..num_transcoder_written]).map_err(map_write_err)?;
        if result == enc::CoderResult::InputEmpty {
            break;
        }
    }
    return Ok(());
}

fn map_write_err(err: io::Error) -> error::TranscodeError {
    return error::TranscodeError::Write(err);
}

fn map_read_err(err: io::Error) -> error::TranscodeError {
    return error::TranscodeError::Read(err);
}

// fn _exit_with_io_error(message: &str, cause: io::Error) {
//     eprintln!("{}: {}", message, cause);
//     std::process::exit(constants::IO_ERROR);
// }

#[cfg(test)]
mod tests {
    use std::path;
    use std::io::Read;

    macro_rules! test_transcode {
        ($name:ident, $input_file:expr, $expected_file:expr, $enc:expr) => {
            #[test]
            fn $name() {
                let mut opt = super::option::Opt::default();
                *(opt.chars_to_guess_mut()) = 100;
                let test_data = path::Path::new("test_data");
                let ifile_handle = &mut std::fs::File::open(test_data.join($input_file)).unwrap();
                let enc = super::enc::Encoding::for_label($enc.as_bytes()).unwrap_or(&super::enc::UTF_8_INIT);
                let output = &mut Vec::with_capacity(20*1024);
                let input_buffer = &mut [0u8; 5*1024]; // 5K bytes
                // let input_buffer = &mut [0u8; 32]; // 5K bytes
                let output_buffer = &mut [0u8; 10*1024]; // 10K bytes
                // let output_buffer = &mut [0u8; 128]; // 10K bytes
                let _ = super::transcode(ifile_handle, output, enc, input_buffer, output_buffer, &opt);
                let test_data = path::Path::new("test_data");
                let efile_handle = &mut std::fs::File::open(test_data.join($expected_file)).unwrap();
                let expected_string = &mut Vec::with_capacity(20*1024);
                efile_handle.read_to_end(expected_string).unwrap();
                let src_encoding_name = $input_file.split_once('_').unwrap_or_else(|| $input_file.split_once('.').unwrap()).0;
                let ofile_name = String::new()+$expected_file+"."+src_encoding_name+".output";
                let ofile_handle: &mut dyn std::io::Write
                    = &mut std::fs::File::create(test_data.join(ofile_name)).unwrap();
                ofile_handle.write_all(output).unwrap();
                assert!(output == expected_string);
            }
        };
    }
    test_transcode!(transcode_utf16le_utf8     , "utf16le_BOM_th.txt"  , "utf8_th.txt"     , "utf8");
    test_transcode!(transcode_utf16be_utf8     , "utf16be_BOM_th.txt"  , "utf8_th.txt"     , "utf8");
    test_transcode!(transcode_sjis_utf8        , "sjis_ja.txt"         , "utf8_ja.txt"     , "utf8");
    test_transcode!(transcode_eucjp_utf8       , "euc-jp_ja.txt"       , "utf8_ja.txt"     , "utf8");
    test_transcode!(transcode_iso2022jp_utf8   , "iso-2022-jp_ja.txt"  , "utf8_ja.txt"     , "utf8");
    test_transcode!(transcode_big5_utf8        , "big5_zh_CHT.txt"     , "utf8_zh_CHT.txt" , "utf8");
    test_transcode!(transcode_gbk_utf8         , "gbk_zh_CHS.txt"      , "utf8_zh_CHS.txt" , "utf8");
    test_transcode!(transcode_gb18030_utf8     , "gb18030_zh_CHS.txt"  , "utf8_zh_CHS.txt" , "utf8");
    test_transcode!(transcode_euckr_utf8       , "euc-kr_ko.txt"       , "utf8_ko.txt"     , "utf8");
    test_transcode!(transcode_koi8r_utf8       , "koi8-r_ru.txt"       , "utf8_ru.txt"     , "utf8");
    test_transcode!(transcode_windows1252_utf8 , "windows-1252_es.txt" , "utf8_es.txt"     , "utf8");
    test_transcode!(transcode_ascii_utf8       , "ascii_en.txt"        , "utf8_en.txt"     , "utf8");

    test_transcode!(transcode_utf8_utf16le     , "utf8_th.txt"     , "utf16le_BOM_th.txt"  , "utf-16le"     );
    test_transcode!(transcode_utf8_utf16be     , "utf8_th.txt"     , "utf16be_BOM_th.txt"  , "utf-16be"     );
    test_transcode!(transcode_utf8_sjis        , "utf8_ja.txt"     , "sjis_ja.txt"         , "sjis"         );
    test_transcode!(transcode_utf8_eucjp       , "utf8_ja.txt"     , "euc-jp_ja.txt"       , "euc-jp"       );
    test_transcode!(transcode_utf8_iso2022jp   , "utf8_ja.txt"     , "iso-2022-jp_ja.txt"  , "iso-2022-jp"  );
    test_transcode!(transcode_utf8_big5        , "utf8_zh_CHT.txt" , "big5_zh_CHT.txt"     , "big5"         );
    test_transcode!(transcode_utf8_gbk         , "utf8_zh_CHS.txt" , "gbk_zh_CHS.txt"      , "gbk"          );
    test_transcode!(transcode_utf8_gb18030     , "utf8_zh_CHS.txt" , "gb18030_zh_CHS.txt"  , "gb18030"      );
    test_transcode!(transcode_utf8_euckr       , "utf8_ko.txt"     , "euc-kr_ko.txt"       , "euc-kr"       );
    test_transcode!(transcode_utf8_koi8r       , "utf8_ru.txt"     , "koi8-r_ru.txt"       , "koi8-r"       );
    test_transcode!(transcode_utf8_windows1252 , "utf8_es.txt"     , "windows-1252_es.txt" , "windows-1252" );
    test_transcode!(transcode_utf8_ascii       , "utf8_en.txt"     , "ascii_en.txt"        , "ascii"        );

    test_transcode!(transcode_no_encoding_binary , "binary.jpeg"     , "binary.jpeg"         , "binary"     );
    test_transcode!(transcode_no_encoding_utf8   , "utf8_th.txt"     , "utf8_th.txt"         , "utf8"       );
}
