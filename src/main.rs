#![feature(test)]

mod args;
mod lib;

use std::error::Error;
use std::io::Write;

fn print_usage(program: &str, opts: &args::Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn write_error(message: &str) {
    writeln!(&mut std::io::stderr(), "{}", &message).expect("Failed to write to stderr.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let opts = args::build_opts();
    let (filename, number, min_entropy, ngram_length, min_word_length) =
        match args::parse_args(&opts, &args) {
            Ok(parsed_args) => parsed_args,
            Err(_) => {
                print_usage(program, &opts);
                return;
            }
        };

    let ngrams = match lib::get_ngrams(
        filename.as_ref().map(String::as_ref),
        ngram_length,
        min_word_length,
    ) {
        Ok(ngrams) => ngrams,
        Err(_) => {
            let filename_str = filename.unwrap_or("-".to_string());
            write_error(&format!(
                "{}: {}: {}",
                program, &filename_str, "Failed to read input."
            ));
            return;
        }
    };
    let passphrases = match lib::gen_passphrases(ngrams, number, min_entropy) {
        Ok(passphrases) => passphrases,
        Err(e) => {
            write_error(&format!("{}: {}", program, e.description()));
            return;
        }
    };

    for (passphrase, entropy) in passphrases {
        println!("{} <{}>", passphrase, entropy);
    }
}
