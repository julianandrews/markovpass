extern crate getopts;
extern crate markovpass;

use markovpass::PassphraseMarkovChain;
use std::io::Write;

fn clean_word(word: &str, min_length: usize) -> Option<&str> {
    let word = word.trim_matches(|c: char| !c.is_alphabetic());
    if word.chars().all(|c: char| c.is_alphabetic()) && word.len() >= min_length {
        Some(word)
    } else {
        None
    }
}

fn get_ngrams(corpus: &str, ngram_length: usize, min_word_length: usize) -> Vec<String> {
    let words = corpus.split_whitespace()
      .filter_map(|word| clean_word(word, min_word_length));
    let cleaned_corpus = Some("").into_iter().chain(words).collect::<Vec<&str>>().join(" ");
    let char_count = cleaned_corpus.chars().count();
    if char_count < ngram_length { return vec![]; };
    let count = char_count - ngram_length + 1;
    let mut chars = cleaned_corpus.chars();

    let mut ngrams = Vec::with_capacity(count);
    for _ in 0..count {
        let ngram: String = chars.clone().take(ngram_length).collect();
        ngrams.push(ngram.to_lowercase());
        chars.next();
    };
    ngrams
}

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut opts = getopts::Options::new();
    let program = args[0].clone();

    opts.optopt("n", "", "Number of passphrases to generate (default 1)", "NUM");
    opts.optopt("e", "", "Minimum entropy (default 60)", "MINENTROPY");
    opts.optopt("l", "", "NGram length (default 3)", "LENGTH");
    opts.optopt("w", "", "Minimum word length for corpus (default 5)", "LENGTH");
    opts.optflag("h", "help", "display this help and exit");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(_) => {
            print_usage(&program, opts);
            return;
        },
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    };

    macro_rules! get_usize_flag {
        ($flag:expr, $default:expr) => {
            match matches.opt_str($flag) {
                Some(s) => match s.parse::<usize>() {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage(&program, opts);
                        return;
                    },
                },
                None => $default,
            }
        }
    }

    let number = get_usize_flag!("n", 1);
    let min_entropy = get_usize_flag!("e", 60);
    let ngram_length = get_usize_flag!("l", 3);
    let min_word_length = get_usize_flag!("w", 5);

    if matches.free.len() > 1 {
        print_usage(&program, opts);
        return;
    };

    let filename = if matches.free.is_empty() {
        "-".to_string()
    } else {
        matches.free[0].clone()
    };

    macro_rules! write_error {
        ($message:expr) => {
            let r = writeln!(&mut std::io::stderr(), "{}: {}: {}", &program, &filename, $message);
            r.expect("failed printing to stderr");
        }
    }

    let mut input: Box<std::io::Read> = if filename == "-" {
        Box::new(std::io::stdin())
    } else {
        let file = match std::fs::File::open(&filename) {
            Ok(file) => file,
            Err(_) => {
                write_error!("Failed to read input.");
                return;
            },
        };
        Box::new(file)
    };

    let mut corpus = String::new();
    match input.read_to_string(&mut corpus) {
        Err(_) => {
            write_error!("Failed to read input.");
            return;
        },
        Ok(_) => {},
    }

    let ngrams = get_ngrams(&corpus, ngram_length, min_word_length);
    if ngrams.is_empty() {
        write_error!("No NGrams found.");
        return;
    };
    let chain = match PassphraseMarkovChain::new(ngrams.iter().cloned()) {
        Ok(chain) => chain,
        Err(e) => {
            write_error!(&e);
            return;
        }
    };

    for _ in 0..number {
        let (passphrase, entropy) = chain.passphrase(min_entropy as f64);
        println!("{} <{}>", passphrase, entropy);
    }
}
