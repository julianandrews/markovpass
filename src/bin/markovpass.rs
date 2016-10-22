extern crate getopts;
extern crate markovpass;

use markovpass::*;
use std::error::Error;
use std::io::Write;

fn build_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optopt("n", "", "Number of passphrases to generate (default 1)", "NUM");
    opts.optopt("e", "", "Minimum entropy (default 60)", "MINENTROPY");
    opts.optopt("l", "", "NGram length (default 3, must be > 1)", "LENGTH");
    opts.optopt("w", "", "Minimum word length for corpus (default 5)", "LENGTH");
    opts.optflag("h", "help", "Display this help and exit");

    opts
}

fn parse_flag<T: std::str::FromStr>(matches: &getopts::Matches, flag: &str, default: T)
        -> Result<T, &'static str> {
    matches.opt_str(flag)
        .map(|c| c.parse::<T>())
        .unwrap_or(Ok(default))
        .map_err(|_| "Failed to parse flag.")
}

fn parse_args(opts: &getopts::Options, args: &Vec<String>)
        -> Result<(Option<String>, usize, f64, usize, usize), &'static str> {
    let matches = try!(opts.parse(&args[1..]).map_err(|_| "Failed to parse arguments."));

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
        Some(matches.free[0].clone())
    };

    Ok((filename, number, min_entropy, ngram_length, min_word_length))
}

fn get_corpus(filename: Option<&str>) -> Result<String, std::io::Error> {
    let mut input: Box<std::io::Read> = match filename {
        Some(filename) => Box::new(try!(std::fs::File::open(&filename))),
        None => Box::new(std::io::stdin()),
    };
    let mut corpus = String::new();
    try!(input.read_to_string(&mut corpus));

    Ok(corpus)
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || c == '\''
}

fn clean_word(word: &str, min_length: usize) -> Option<&str> {
    let word = word.trim_matches(|c| ! is_word_char(c));

    if word.chars().all(is_word_char) && word.len() >= min_length {
        Some(word)
    } else {
        None
    }
}

fn get_ngrams(corpus: String, ngram_length: usize, min_word_length: usize) -> Vec<String> {
    let corpus = corpus.to_lowercase();
    let words = corpus.split_whitespace()
      .filter_map(|word| clean_word(word, min_word_length));
    let cleaned_corpus = Some("").into_iter().chain(words).collect::<Vec<&str>>().join(" ");
    let count = cleaned_corpus.chars().count();
    if count < ngram_length { return vec![]; };

    let mut ngrams = Vec::with_capacity(count);

    let ngrams_it = ToNgrams::new(ngram_length, cleaned_corpus);
    for ngram in ngrams_it {
        ngrams.push(ngram.to_string());
    }

    ngrams
}

fn gen_passphrases(ngrams: Vec<String>, number: usize, min_entropy: f64)
        -> Result<Vec<(String, f64)>, PassphraseMarkovChainError> {
    let chain = try!(PassphraseMarkovChain::new(ngrams.iter().cloned()));
    let mut passphrases = Vec::with_capacity(number);
    for _ in 0..number {
        passphrases.push(chain.passphrase(min_entropy));
    };

    Ok(passphrases)
}

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn write_error(message: &str) {
    writeln!(&mut std::io::stderr(), "{}", &message).expect("Failed to write to stderr.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    let opts = build_opts();

    let (filename, number, min_entropy, ngram_length, min_word_length) =
            match parse_args(&opts, &args) {
        Ok(parsed_args) => { parsed_args },
        Err(_) => {
            print_usage(&program, &opts);
            return;
        }
    };
    let corpus = match get_corpus(filename.as_ref().map(String::as_ref)) {
        Ok(corpus) => { corpus },
        Err(_) => {
            let filename_str = filename.unwrap_or("-".to_string());
            write_error(&format!("{}: {}: {}", &program, &filename_str, "Failed to read input."));
            return;
        },
    };
    let ngrams = get_ngrams(corpus, ngram_length, min_word_length);
    let passphrases = match gen_passphrases(ngrams, number, min_entropy) {
        Ok(passphrases) => { passphrases },
        Err(e) => {
            write_error(&format!("{}: {}", &program, e.description()));
            return;
        },
    };

    for (passphrase, entropy) in passphrases {
        println!("{} <{}>", passphrase, entropy);
    };
}

#[cfg(test)]
mod tests {
    use super::{clean_word,get_ngrams,gen_passphrases};

    #[test]
    fn test_clean_word() {
        assert_eq!(clean_word("Test", 3), Some("Test"));
        assert_eq!(clean_word("123test@314", 3), Some("test"));
        assert_eq!(clean_word("2#@test'in23", 3), Some("test'in"));
        assert_eq!(clean_word("31ld;Test", 3), None);
        assert_eq!(clean_word("a", 2), None);
        assert_eq!(clean_word("Test", 5), None);
    }

    #[test]
    fn test_get_ngrams() {
        assert_eq!(
            get_ngrams("this is a test".to_string(), 3, 3),
            vec! [" th", "thi", "his", "is ", "s t", " te", "tes", "est", "st ", "t t"]
        );
        assert_eq!(
            get_ngrams("this is a test".to_string(), 5, 3),
            vec! [" this", "this ", "his t", "is te", "s tes", " test",
                  "test ", "est t", "st th", "t thi"]
        );
        assert_eq!(
            get_ngrams("this is a test".to_string(), 3, 2),
            vec! [" th", "thi", "his", "is ", "s i", " is", "is ", "s t",
                  " te", "tes", "est", "st ", "t t"]
        );
        assert!(get_ngrams("this is a test".to_string(), 3, 5).is_empty(), "Ngrams not empty");
        assert_eq!(
            get_ngrams("Some awes0me test".to_string(), 6, 3),
            vec! [" some ", "some t", "ome te", "me tes", "e test", " test ",
                  "test s", "est so", "st som", "t some"]
        );
        assert_eq!(
            get_ngrams("test'in".to_string(), 3, 3),
            vec! [" te", "tes", "est", "st'", "t'i", "'in", "in ", "n t"]
        );
    }

    #[test]
    fn test_gen_passphrases() {
        let ngrams = get_ngrams("tic toc".to_string(), 3, 3);
        let min_entropy = 60.0;
        let result = gen_passphrases(ngrams, 5, min_entropy);
        assert!(result.is_ok(), "Passphrase generation failed.");
        let passphrases = result.unwrap();
        assert_eq!(passphrases.len(), 5);
    }

    #[test]
    fn test_gen_passphrases_no_ngrams() {
        let ngrams: Vec<String> = vec![];
        let result = gen_passphrases(ngrams, 1, 60.0);
        assert!(result.is_err(), "No error despite no ngrams.");
    }

    #[test]
    fn test_gen_passphrases_no_entropy() {
        let ngrams = get_ngrams("abc def ghijkl mnopqr stuvw xyz".to_string(), 2, 1);
        let result = gen_passphrases(ngrams, 1, 60.0);
        assert!(result.is_err(), "No error despite no entropy.");
    }

    #[test]
    fn test_gen_passphrases_no_starting_entropy() {
        let ngrams = get_ngrams("tictoctictactoe".to_string(), 2, 1);
        let result = gen_passphrases(ngrams, 1, 60.0);
        assert!(result.is_err(), "No error despite no starting entropy.");
    }
}
