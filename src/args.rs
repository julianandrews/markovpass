extern crate getopts;

use std::path::PathBuf;

pub fn build_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optopt(
        "n",
        "",
        "Number of passphrases to generate (default 1)",
        "NUM",
    );
    opts.optopt("e", "", "Minimum entropy (default 60)", "MINENTROPY");
    opts.optopt("l", "", "NGram length (default 3, must be > 1)", "LENGTH");
    opts.optopt(
        "w",
        "",
        "Minimum word length for corpus (default 5)",
        "LENGTH",
    );
    opts.optflag("h", "help", "Display this help and exit");

    opts
}

pub fn parse_args(
    opts: &getopts::Options,
    args: &Vec<String>,
) -> Result<(Option<PathBuf>, usize, f64, usize, usize), &'static str> {
    let matches = try!(
        opts.parse(&args[1..])
            .map_err(|_| "Failed to parse arguments.")
    );

    if matches.opt_present("h") || matches.free.len() > 1 {
        return Err("Failed to parse arguments.");
    };

    let number = try!(parse_flag(&matches, "n", 1));
    let min_entropy = try!(parse_flag(&matches, "e", 60.0));
    let ngram_length = try!(parse_flag(&matches, "l", 3));
    let min_word_length = try!(parse_flag(&matches, "w", 5));

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

fn parse_flag<T: ::std::str::FromStr>(
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
