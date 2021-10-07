use std::io;
use std::env;
use structopt::StructOpt;
use std::path::PathBuf;

/// Converts texts from an auto detected encoding to utf-8 or a specified encoding.  
/// If malformed byte sequences are found, they are replaced with REPLACEMENT CHARACTER(U+FFFD).  
/// If the auto-detection is considered it failed, the input texts are output as-is, meaning no conversion takes place, with an error message emitted.  
#[derive(StructOpt, Debug)]
#[structopt(name = "8fy")]
struct Opt {
    /// Print version info and exit
    #[structopt(long)]
    version: bool,

    /// The encoding of the output.
    /// Default is utf-8.
    #[structopt(short, long)]
    to_code: String,

    /// Output directory.
    /// If input arguments contain directories, the directory hierarchies are preserved under DIRECTORY.
    #[structopt(short, long, parse(from_os_str))]
    output_directory: PathBuf,

    /// Print supported encodings.
    #[structopt(short, long)]
    list: bool,

    /// Threshold (0-100) of non-text characters occurrence.
    /// Above this threshold in decoded texts, the auto-detection is treated as it failed.
    /// In that case the input texts are output as-is with an error message emitted.
    /// The default value is 0.
    #[structopt(short, long, default_value = "0")]
    non_text_threshold: u8,

    /// Print only auto-detected encodings without decoded texts.
    #[structopt(short, long)]
    encoding: bool,

    /// Suppress error messages.
    #[structopt(short, long)]
    quiet: bool,

    /// Files (or directories) to process
    #[structopt(name = "FILE(DIRECTORY)", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}

