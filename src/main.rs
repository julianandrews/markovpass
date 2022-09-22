#![cfg_attr(feature = "benchmarks", feature(test))]

mod lib;

use clap::{AppSettings, Parser};

fn main() {
    let args = Args::parse();

    let gen_passphrase_options = lib::GenPassphraseOptions {
        filename: args.filename,
        number: args.number,
        min_entropy: args.min_entropy,
        ngram_length: args.ngram_length,
        min_word_length: args.min_word_length,
    };
    let passphrases = match lib::gen_passphrases(&gen_passphrase_options) {
        Ok(passphrases) => passphrases,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    for (passphrase, entropy) in passphrases {
        if args.show_entropy {
            println!("{} <{}>", passphrase, entropy);
        } else {
            println!("{}", passphrase);
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, setting = AppSettings::DeriveDisplayOrder)]
struct Args {
    /// Markovchain corpus
    #[clap(value_parser)]
    pub filename: Option<std::path::PathBuf>,

    /// Number of passphrases to generate
    #[clap(short = 'n', value_parser, default_value_t = 1)]
    pub number: usize,

    /// Minimum entropy
    #[clap(short = 'e', value_parser, default_value_t = 60.0)]
    pub min_entropy: f64,

    /// Ngram length
    #[clap(short = 'l', value_parser, default_value_t = 3)]
    pub ngram_length: usize,

    /// Minimum word length for corpus
    #[clap(short = 'w', value_parser, default_value_t = 5)]
    pub min_word_length: usize,

    /// Print the entropy for each passphrase
    #[clap(long, value_parser, default_value_t = false)]
    pub show_entropy: bool,
}
