#![cfg_attr(feature = "benchmarks", feature(test))]

mod args;
mod lib;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opts = args::build_opts();
    let parsed_args = match args::parse_args(&opts, &args) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{}", error);
            args::print_usage(&args[0], &opts);
            std::process::exit(1);
        }
    };

    if parsed_args.print_help {
        args::print_usage(&args[0], &opts);
        return;
    }

    let gen_passphrase_options = lib::GenPassphraseOptions {
        filename: parsed_args.filename,
        number: parsed_args.number,
        min_entropy: parsed_args.min_entropy,
        ngram_length: parsed_args.ngram_length,
        min_word_length: parsed_args.min_word_length,
    };
    let passphrases = match lib::gen_passphrases(&gen_passphrase_options) {
        Ok(passphrases) => passphrases,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    };

    for (passphrase, entropy) in passphrases {
        if parsed_args.show_entropy {
            println!("{} <{}>", passphrase, entropy);
        } else {
            println!("{}", passphrase);
        }
    }
}
