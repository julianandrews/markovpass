extern crate getopts;
extern crate markovpass;

use markovpass::PassphraseMarkovChain;
use std::io::Write;

fn build_opts() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optopt("n", "", "Number of passphrases to generate (default 1)", "NUM");
    opts.optopt("e", "", "Minimum entropy (default 60)", "MINENTROPY");
    opts.optopt("l", "", "NGram length (default 3)", "LENGTH");
    opts.optopt("w", "", "Minimum word length for corpus (default 5)", "LENGTH");
    opts.optflag("h", "help", "display this help and exit");

    opts
}

fn parse_args(opts: &getopts::Options, args: &Vec<String>)
        -> Result<(String, usize, f64, usize, usize), &'static str> {
    let matches = match opts.parse(&args[1..]) {
        Ok(x) => { x },
        Err(_) => { return Err("Failed to parse arguments."); },
    };

    if matches.opt_present("h") || matches.free.len() > 1 {
        return Err("Failed to parse arguments.");
    };

    macro_rules! get_num_flag {
        ($num_type:ty, $flag:expr, $default:expr) => {
            match matches.opt_str($flag) {
                Some(s) => match s.parse::<$num_type>() {
                    Ok(n) => { n },
                    Err(_) => { return Err("Failed to parse arguments."); }
                },
                None => $default,
            }
        }
    }

    let number = get_num_flag!(usize, "n", 1);
    let min_entropy = get_num_flag!(f64, "e", 60.0);
    let ngram_length = get_num_flag!(usize, "l", 3);
    let min_word_length = get_num_flag!(usize, "w", 5);

    let filename = if matches.free.is_empty() {
        "-".to_string()
    } else {
        matches.free[0].clone()
    };

    Ok((filename, number, min_entropy, ngram_length, min_word_length))
}

fn get_corpus(filename: &str) -> Result<String, std::io::Error> {
    let mut input: Box<std::io::Read> = if filename == "-" {
        Box::new(std::io::stdin())
    } else {
        let file = try!(std::fs::File::open(&filename));
        Box::new(file)
    };
    let mut corpus = String::new();
    try!(input.read_to_string(&mut corpus));

    Ok(corpus)
}

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

fn gen_passphrases(ngrams: Vec<String>, number: usize, min_entropy: f64)
        -> Result<Vec<(String, f64)>, &'static str> {
    if ngrams.is_empty() { return Err("No NGrams found."); };
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
    if ngram_length < 2 {
        write_error("Ngram length must be greater than one.");
    };
    let corpus = match get_corpus(&filename) {
        Ok(corpus) => { corpus },
        Err(_) => {
            write_error(&format!("{}: {}: {}", &program, &filename, "Failed to read input."));
            return;
        },
    };
    let ngrams = get_ngrams(&corpus, ngram_length, min_word_length);
    let passphrases = match gen_passphrases(ngrams, number, min_entropy) {
        Ok(passphrases) => { passphrases },
        Err(e) => {
            write_error(&format!("{}: {}: {}", &program, &filename, e));
            return;
        },
    };

    for (passphrase, entropy) in passphrases {
        println!("{} <{}>", passphrase, entropy);
    };
}
