pub mod option;
pub mod constants;
pub mod transcoder;

use encoding_rs as enc;
use std::io;
use std::fs;
use std::str;
use std::process;

pub fn cli(opt: option::Opt) {
    // check
    let mut file = match opt.paths.iter().next() {
        Some(path) => fs::File::open(path).unwrap(),
        None => return,
    };

    let mut stdout = std::io::stdout();
    let to_code = enc::Encoding::for_label(opt.to.as_bytes()).expect("Unsupported encoding.");
    controller(&mut file, &mut stdout, to_code, opt);
}

pub fn controller(read: &mut impl io::Read, write: &mut impl io::Write, encoding: &'static enc::Encoding, opt: option::Opt) {

    let input_buffer = &mut [0u8; 5 * (1 << 10)]; // 5K bytes
    let decode_buffer = &mut [0u8; 15 * (1 << 10)]; // 15K bytes
    let output_buffer = &mut [0_u8; 10 * (1 << 10)]; // 10K bytes

    // guess the input encoding using up to a few Kbytes of byte sequences
    let (mut buf_guess, eof) = {
        let mut buf_eof = [0; 1]; // buffer to check if eof
        let buf_size=read.read(input_buffer).unwrap_or(0);
        let buf_eof_size=read.read(&mut buf_eof).unwrap_or(0);
        let buf_guess = [&input_buffer[..buf_size], &buf_eof[..buf_eof_size]].concat();
        let eof = buf_eof_size == 0;
        (buf_guess, eof)
    };
    let num_non_aschii = 1000; // 1000 chars of non aschii
    let transcoder = &mut transcoder::Transcoder::new_with_buff_size(None, encoding, 10 * 1024).unwrap();
    let num_read = {
        let rst = transcoder.guess_and_transcode(&mut buf_guess, output_buffer, eof, num_non_aschii).and_then(|x| {
            if transcoder.src_encoding().unwrap() == transcoder.dst_encoding() {
                Err("".into())
            } else {
                Ok(x)
            }
        });
        match rst {
            Ok((_, num_read, num_written)) => {
                // write transcoded bytes in buffer
                try_write(|| write.write_all(&output_buffer[..num_written]));
                num_read
            },
            Err(err) => {
                if err != "" && !opt.quiet {
                    eprintln!("Auto detection failed");
                }
                try_write(|| write.write_all(&buf_guess));
                try_write(|| io::copy(read, write).map(|_| ()));
                return;
            },
        }
    };

    // decode rest of bytes in buffer
    transcode_buffer_and_write(write, transcoder, &input_buffer[num_read..], output_buffer, false);

    // decode bytes remaining in file
    transcode_file_and_write(read, write, transcoder, input_buffer, output_buffer);
}

fn transcode_file_and_write(read: &mut impl io::Read,write: &mut impl io::Write, transcoder: &mut transcoder::Transcoder,
    input_buffer: &mut [u8], output_buffer: &mut [u8]) {
    loop {
        match read.read(input_buffer) {
            Ok(num_read) => {
                let eof = num_read == 0;
                transcode_buffer_and_write(write, transcoder, input_buffer, output_buffer, eof);
                if eof {
                    break;
                }
            }
            Err(cause) => {
                exit_with_io_error("Error reading input", cause);
            }
        }
    };
}

fn transcode_buffer_and_write(write: &mut impl io::Write, transcoder: &mut transcoder::Transcoder,
    src: &[u8], output_buffer: &mut [u8], eof: bool) {
    let mut transcoder_input_start = 0;
    loop {
        let (result, num_transcoder_read, num_transcoder_written)
            = transcoder.transcode(&src[transcoder_input_start..], output_buffer, eof);
        transcoder_input_start+=num_transcoder_read;
        try_write(|| write.write_all(&output_buffer[..num_transcoder_written]));
        if result == enc::CoderResult::InputEmpty {
            break;
        }
    }
}

fn try_write(mut fnc: impl FnMut() -> Result<(),io::Error>) {
    if let Err(cause) = fnc() {
        exit_with_io_error("Error writing output", cause);
    }
}

fn exit_with_io_error(message: &str, cause: io::Error) {
    eprintln!("{}: {}", message, cause);
    process::exit(constants::IO_ERROR);
}

#[cfg(test)]
mod tests {
    use std::path;

    // #[test]
    // fn it_works() {
    //     let s = "hoge";
    //     let t = s;
    //     println!("{}, {}",s, t);
    //     assert_eq!(2 + 2, 4);
    //     // println!("{}",1 << 10);
    //     // println!("{}",2_i32.pow(10));
    //     super::guess();
    // }
    #[test]
    fn gif() {
        let mut opt = super::option::Opt::default();
        let path = path::PathBuf::from("src/test_data/demo.gif");
        opt.paths.push(path);
        // let (fuga, success) = super::guess(opt).expect("couldn't guess");
        // println!("{:?}, {:?}", success, fuga);
        // let buf = fuga.decode(buf_slice);
    }
}

