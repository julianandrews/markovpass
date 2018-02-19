#![feature(test)]

mod args;
mod lib;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let opts = args::build_opts();
    let (filename, number, min_entropy, ngram_length, min_word_length) =
        match args::parse_args(&opts, &args) {
            Ok(parsed_args) => parsed_args,
            Err(_) => {
                args::print_usage(program, &opts);
                return;
            }
        };

    let passphrases =
        match lib::gen_passphrases(filename, number, min_entropy, ngram_length, min_word_length) {
            Ok(passphrases) => passphrases,
            Err(e) => {
                eprintln!("{}: {}", program, e.description());
                return;
            }
        };

    for (passphrase, entropy) in passphrases {
        println!("{} <{}>", passphrase, entropy);
    }
}
