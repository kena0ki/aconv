use structopt::StructOpt;
use std::path::PathBuf;

/// Converts texts from the auto-detected encoding to UTF-8 or a specified encoding.
/// If byte sequences that is malformed as Unicode are found,
/// they are replaced with the REPLACEMENT CHARACTER(U+FFFD).
/// If the destination encoding is not Unicode and unmappable characters are found, they are
/// replaced with the corresponding numeric character references.
/// If the encoding detection is considered it failed, the input texts are output as-is,
/// meaning no conversion takes place, and an error message is emitted.
#[derive(StructOpt, Debug, Default)]
#[structopt(verbatim_doc_comment, version=env!("CARGO_PKG_VERSION"))]
pub struct Opt {
    /// Prints version information.
    #[structopt(short="V", long)]
    pub version: bool,

    /// The encoding of the output.
    #[structopt(name = "ENCODING", short = "t", long = "to-code", default_value = "UTF-8")]
    pub to_code: String,

    /// Output directory.
    /// If input arguments contain directories, the directory hierarchies are preserved under DIRECTORY.
    #[structopt(name = "DIRECTORY", short = "o", long = "output", parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Prints supported encodings.
    #[structopt(short, long)]
    pub list: bool,

    /// The threshold (0-100) of non-text character occurrence.
    /// Above this threshold in decoded UTF-8 texts, the encoding detection is treated as it failed.
    /// In that case the input texts are output as-is with an error message emitted.
    #[structopt(name = "PERCENTAGE", short = "T", long = "non-text-threshold", default_value = "0")]
    pub non_text_threshold: u8,

    /// The number of non-ASCII characters to guess the encoding.
    /// Around 100 characters are enough for most cases, but if the guess is not accurate, increasing the value
    /// might help.
    #[structopt(name = "NUMBER", short = "A", long = "non_ascii_to_guess", default_value = "100")]
    pub non_ascii_to_guess: usize,

    /// Only shows auto-detected encodings without decoded texts.
    #[structopt(short, long)]
    pub show: bool,

    /// Suppresses error messages when encoding detection failed.
    #[structopt(short, long)]
    pub quiet: bool,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    pub paths: Vec<PathBuf>,
}

impl Opt {
    pub fn new() -> Self {
        let mut opt = Opt::default();
        opt.non_ascii_to_guess = 100;
        opt.to_code = "UTF-8".into();
        return opt;
    }
}


