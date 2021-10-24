use crate::option;
use crate::error;

use transcoding_rs as tc;
use encoding_rs as enc;
use std::io;

pub fn transcode(reader: &mut dyn io::Read, writer: &mut dyn io::Write, encoding: &'static enc::Encoding, opt: &option::Opt)
    -> Result<&'static enc::Encoding, error::TranscodeError> {

    let detector = tc::I18nReaderEncodingDetector::new()
        .buffer_size(10 * 1024)
        .non_ascii_to_guess(opt.non_ascii_to_guess)
        .non_text_threshold(opt.non_text_threshold)
        .add_bom_utf16(true);
    let guess_result = detector.guess_with_dst_encoding(reader, encoding).map_err(map_read_err)?;
    match guess_result {
        tc::GuessResult::NoInput => {
            writer.write_all(&[]).map_err(map_write_err)?;
            return Ok(enc::UTF_8);
        },
        tc::GuessResult::Success(mut i18n_reader, enc) => {
            if opt.show {
                return Ok(enc);
            }
            io::copy(&mut i18n_reader, writer).map(|_| ()).map_err(map_write_err)?;
            return Ok(enc);
        },
        tc::GuessResult::Fail(mut i18n_reader) => { // if no encoding is found
            // write input to output as-is
            io::copy(&mut i18n_reader, writer).map(|_| ()).map_err(map_write_err)?;
            return Err(error::TranscodeError::Guess("Auto-detection seems to fail.".into()));
        }
    }
}

fn map_write_err(err: io::Error) -> error::TranscodeError {
    return error::TranscodeError::Write(err);
}

fn map_read_err(err: io::Error) -> error::TranscodeError {
    return error::TranscodeError::Read(err);
}

#[cfg(test)]
mod tests {
    use std::path;
    use std::io::Read;

    macro_rules! test_transcode {
        ($name:ident, $input_file:expr, $expected_file:expr, $enc:expr) => {
            #[test]
            fn $name() {
                let opt = super::option::Opt::new();
                let test_data = path::Path::new("test_data");
                let ifile_handle = &mut std::fs::File::open(test_data.join($input_file)).unwrap();
                let enc = super::enc::Encoding::for_label($enc.as_bytes()).unwrap_or(&super::enc::UTF_8_INIT);
                let output = &mut Vec::with_capacity(20*1024);
                let _ = super::transcode(ifile_handle, output, enc, &opt);
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

