extern crate getopts;

use std::path::PathBuf;

const DEFAULT_NUMBER: usize = 1;
const DEFAULT_MIN_ENTROPY: f64 = 60.0;
const DEFAULT_NGRAM_LENGTH: usize = 3;
const DEFAULT_MIN_WORD_LENGTH: usize = 5;

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

    opts
}

pub fn parse_args(
    opts: &getopts::Options,
    args: &Vec<String>,
) -> Result<(Option<PathBuf>, usize, f64, usize, usize), &'static str> {
    let matches = opts
        .parse(&args[1..])
        .map_err(|_| "Failed to parse arguments.")?;

    if matches.opt_present("h") || matches.free.len() > 1 {
        return Err("Failed to parse arguments.");
    };

    let number = parse_flag_or_default(&matches, "n", DEFAULT_NUMBER)?;
    let min_entropy = parse_flag_or_default(&matches, "e", DEFAULT_MIN_ENTROPY)?;
    let ngram_length = parse_flag_or_default(&matches, "l", DEFAULT_NGRAM_LENGTH)?;
    let min_word_length = parse_flag_or_default(&matches, "w", DEFAULT_MIN_WORD_LENGTH)?;

    if ngram_length < 2 {
        return Err("Ngram length must be greater than one.");
    };

    let filename = if matches.free.is_empty() || matches.free[0] == "-" {
        None
    } else {
        Some(PathBuf::from(matches.free[0].clone()))
    };

    Ok((filename, number, min_entropy, ngram_length, min_word_length))
}

pub fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_flag_or_default<T: ::std::str::FromStr>(
    matches: &getopts::Matches,
    flag: &str,
    default: T,
) -> Result<T, &'static str> {
    matches
        .opt_str(flag)
        .map(|c| c.parse::<T>())
        .unwrap_or(Ok(default))
        .map_err(|_| "Failed to parse flag.")
}
