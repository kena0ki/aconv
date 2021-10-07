use structopt::StructOpt;
use std::path::PathBuf;

/// Converts texts from an auto detected encoding to UTF-8 or a specified encoding.
/// If malformed byte sequences are found, they are replaced with REPLACEMENT CHARACTER(U+FFFD).
/// If the auto-detection is considered it failed, the input texts are output as-is,
/// meaning no conversion takes place, with an error message emitted.
#[derive(StructOpt, Debug, Default)]
#[structopt(name = "8fy", verbatim_doc_comment)] // TODO name
pub struct Opt {
    /// Prints version info and exit.
    #[structopt(short, long)]
    pub version: bool,

    /// The encoding of the output.
    #[structopt(name = "ENCODING", short = "t", long = "to-code", default_value = "UTF-8")]
    pub to: String,

    /// Output directory.
    /// If input arguments contain directories, the directory hierarchies are preserved under DIRECTORY.
    #[structopt(name = "DIRECTORY", short = "o", long = "output", parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Prints supported encodings.
    #[structopt(short, long)]
    pub list: bool,

    /// Threshold (0-100) of non-text character occurrence.
    /// Above this threshold in decoded texts, the auto-detection is treated as it failed.
    /// In that case the input texts are output as-is with an error message emitted.
    #[structopt(name = "PERCENTAGE", short = "n", long = "non-text-threshold", default_value = "0")]
    pub non_text_threshold: u8,

    /// Prints only auto-detected encodings without decoded texts.
    #[structopt(short, long)]
    pub encoding: bool,

    /// Suppress error messages.
    #[structopt(short, long)]
    pub quiet: bool,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    pub files: Vec<PathBuf>,
}

