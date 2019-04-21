extern crate getopts;

const DEFAULT_NUMBER: usize = 1;
const DEFAULT_MIN_ENTROPY: f64 = 60.0;
const DEFAULT_NGRAM_LENGTH: usize = 3;
const DEFAULT_MIN_WORD_LENGTH: usize = 5;

use std::fmt;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum UsageError {
    ArgumentParseError,
    TooManyInputsError,
    NgramLengthError,
    FlagParseError,
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UsageError::ArgumentParseError => write!(f, "Failed to parse arguments."),
            UsageError::TooManyInputsError => write!(f, "Too many inputs."),
            UsageError::NgramLengthError => write!(f, "Ngram length must be greater than one."),
            UsageError::FlagParseError => write!(f, "Failed to parse flag."),
        }
    }
}

impl ::std::error::Error for UsageError {}

pub fn build_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optopt(
        "n",
        "",
        &format!(
            "Number of passphrases to generate (default {})",
            DEFAULT_NUMBER
        ),
        "NUM",
    );
    opts.optopt(
        "e",
        "",
        &format!("Minimum entropy (default {})", DEFAULT_MIN_ENTROPY),
        "MINENTROPY",
    );
    opts.optopt(
        "l",
        "",
        &format!(
            "NGram length (default {}, must be > 1)",
            DEFAULT_NGRAM_LENGTH
        ),
        "LENGTH",
    );
    opts.optopt(
        "w",
        "",
        &format!(
            "Minimum word length for corpus (default {})",
            DEFAULT_MIN_WORD_LENGTH
        ),
        "LENGTH",
    );
    opts.optflag("h", "help", "Display this help and exit");
    opts.optflag("", "show-entropy", "Print the entropy for each passphrase");

    opts
}

pub struct MarkovpassArgs {
    pub filename: Option<PathBuf>,
    pub number: usize,
    pub min_entropy: f64,
    pub ngram_length: usize,
    pub min_word_length: usize,
    pub print_help: bool,
    pub show_entropy: bool,
}

pub fn parse_args(
    opts: &getopts::Options,
    args: &Vec<String>,
) -> Result<MarkovpassArgs, UsageError> {
    let matches = opts
        .parse(&args[1..])
        .map_err(|_| UsageError::ArgumentParseError)?;

    // TODO: Add support for multiple file arguments.
    if matches.free.len() > 1 {
        return Err(UsageError::TooManyInputsError);
    };

    let number = parse_flag_or_default(&matches, "n", DEFAULT_NUMBER)?;
    let min_entropy = parse_flag_or_default(&matches, "e", DEFAULT_MIN_ENTROPY)?;
    let ngram_length = parse_flag_or_default(&matches, "l", DEFAULT_NGRAM_LENGTH)?;
    let min_word_length = parse_flag_or_default(&matches, "w", DEFAULT_MIN_WORD_LENGTH)?;
    let print_help = matches.opt_present("h");
    let show_entropy = matches.opt_present("show-entropy");

    if ngram_length <= 1 {
        return Err(UsageError::NgramLengthError);
    };

    let filename = if matches.free.is_empty() || matches.free[0] == "-" {
        None
    } else {
        Some(PathBuf::from(matches.free[0].clone()))
    };

    Ok(MarkovpassArgs {
        filename: filename,
        number: number,
        min_entropy: min_entropy,
        ngram_length: ngram_length,
        min_word_length: min_word_length,
        print_help: print_help,
        show_entropy: show_entropy,
    })
}

pub fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_flag_or_default<T: ::std::str::FromStr>(
    matches: &getopts::Matches,
    flag: &str,
    default: T,
) -> Result<T, UsageError> {
    // TODO: Surface detailed error information.
    matches
        .opt_str(flag)
        .map(|c| c.parse::<T>())
        .unwrap_or(Ok(default))
        .map_err(|_| UsageError::FlagParseError)
}
