use structopt::StructOpt;
use std::path::PathBuf;

/// Converts texts from an auto detected encoding to UTF-8 or a specified encoding.
/// If malformed byte sequences are found, they are replaced with REPLACEMENT CHARACTER(U+FFFD).
/// If the auto-detection is considered it failed, the input texts are output as-is,
/// meaning no conversion takes place, with an error message emitted.
#[derive(StructOpt, Debug, Default)]
#[structopt(verbatim_doc_comment, version=env!("CARGO_PKG_VERSION"))]
pub struct Opt {
    /// Prints version info and exit.
    #[structopt(short, long)]
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

    /// Threshold (0-100) of non-text character occurrence.
    /// Above this threshold in decoded texts, the auto-detection is treated as it failed.
    /// In that case the input texts are output as-is with an error message emitted.
    #[structopt(name = "PERCENTAGE", short = "n", long = "non-text-threshold", default_value = "0")]
    pub non_text_threshold: u8,

    /// Number of non-textual ascii characters to guess the encoding.
    /// Around 100 characters are enough for most cases, but if the guess is not accurate, increasing the value
    /// might help.
    #[structopt(name = "NUMBER", short = "c", long = "chars-to-guess", default_value = "100")]
    pub chars_to_guess: usize,

    /// Only shows auto-detected encodings without decoded texts.
    #[structopt(short, long)]
    pub show: bool,

    /// Suppresses error messages.
    #[structopt(short, long)]
    pub quiet: bool,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    pub paths: Vec<PathBuf>,
}

impl Opt {
    pub fn new() -> Self {
        let mut opt = Opt::default();
        opt.chars_to_guess = 100;
        opt.to_code = "UTF-8".into();
        return opt;
    }
    pub fn version(mut self: Self, version: bool) -> Self {
        self.version = version;
        return self;
    }

    pub fn to_code(mut self: Self, to_code: &str) -> Self {
        self.to_code = to_code.into();
        return self;
    }

    pub fn output(mut self: Self, output: Option<PathBuf>) -> Self {
        self.output = output;
        return self;
    }

    pub fn list(mut self: Self, list: bool) -> Self {
        self.list = list;
        return self;
    }

    pub fn non_text_threshold(mut self: Self, non_text_threshold: u8) -> Self {
        self.non_text_threshold = non_text_threshold;
        return self;
    }

    pub fn chars_to_guess(mut self: Self, chars_to_guess: usize) -> Self {
        self.chars_to_guess = chars_to_guess;
        return self;
    }

    pub fn show(mut self: Self, show: bool) -> Self {
        self.show = show;
        return self;
    }

    pub fn quiet(mut self: Self, quiet: bool) -> Self {
        self.quiet = quiet;
        return self;
    }

    pub fn paths(mut self: Self, paths: Vec<PathBuf>) -> Self {
        self.paths = paths;
        return self;
    }
}


