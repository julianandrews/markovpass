extern crate getopts;
extern crate markovpass;

use markovpass::PassphraseMarkovChain;

fn clean_word(word: &str, min_length: usize) -> Option<&str> {
    let word = word.trim_matches(|c: char| !c.is_alphabetic());
    if word.chars().all(|c: char| c.is_alphabetic()) && word.len() >= min_length {
        Some(word)
    } else {
        None
    }
}

fn get_ngrams(corpus: &str, ngram_length: usize, min_word_length: usize) -> Vec<String> {
    let words: Vec<&str> = Some("").into_iter().chain(
        corpus.split_whitespace().filter_map(|word| clean_word(word, min_word_length))
        ).collect();
    let cleaned_corpus = words.join(" ");
    let count = cleaned_corpus.chars().count() - ngram_length + 1;
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

    let mut input: Box<std::io::Read> = if filename == "-" {
        Box::new(std::io::stdin())
    } else {
        let file = match std::fs::File::open(&filename) {
            Ok(file) => file,
            Err(_) => {
                println!("{}: {}: {:?}", &program, &filename, "Failed to read input");
                return;
            },
        };
        Box::new(file)
    };

    let mut corpus = String::new();
    input.read_to_string(&mut corpus).expect("Failed to read string");

    let ngrams = get_ngrams(&corpus, ngram_length, min_word_length);
    let chain = PassphraseMarkovChain::new(ngrams.iter().cloned());

    for _ in 0..number {
        let (passphrase, entropy) = chain.passphrase(min_entropy as f64);
        println!("{} <{}>", passphrase, entropy);
    }
}
