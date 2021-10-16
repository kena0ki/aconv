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
    /// Around 100 characters are good enough for most cases, but if guess accuracy is not good, increasing the value
    /// might help.
    #[structopt(name = "NUMBER", short = "n", long = "chars-to-guess", default_value = "100")]
    pub chars_to_guess: usize,

    /// Show only auto-detected encodings without decoded texts.
    #[structopt(short, long)]
    pub show: bool,

    /// Suppress error messages.
    #[structopt(short, long)]
    pub quiet: bool,

    /// Files (or directories) to process
    #[structopt(name = "FILE", parse(from_os_str))]
    pub paths: Vec<PathBuf>,
}

impl Opt {
    pub fn version_mut(self: &mut Self) -> &mut bool {
        return &mut self.version;
    }

    pub fn to_code_mut(self: &mut Self) -> &mut String {
        return &mut self.to_code;
    }

    pub fn output_mut(self: &mut Self) -> &mut Option<PathBuf> {
        return &mut self.output;
    }

    pub fn list_mut(self: &mut Self) -> &mut bool {
        return &mut self.list;
    }

    pub fn non_text_threshold_mut(self: &mut Self) -> &mut u8 {
        return &mut self.non_text_threshold;
    }

    pub fn chars_to_guess_mut(self: &mut Self) -> &mut usize {
        return &mut self.chars_to_guess;
    }

    pub fn show_mut(self: &mut Self) -> &mut bool {
        return &mut self.show;
    }

    pub fn quiet_mut(self: &mut Self) -> &mut bool {
        return &mut self.quiet;
    }

    pub fn paths_mut(self: &mut Self) -> &mut Vec<PathBuf> {
        return &mut self.paths;
    }
}


