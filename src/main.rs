#![feature(test)]

mod args;
mod lib;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let opts = args::build_opts();
    let (filename, number, min_entropy, ngram_length, min_word_length) =
        args::parse_args(&opts, &args).map_err(|error| {
            args::print_usage(program, &opts);

            error
        })?;

    let passphrases =
        lib::gen_passphrases(filename, number, min_entropy, ngram_length, min_word_length)
            .map_err(|error| {
                eprintln!("{}: {}", program, error.description());

                error
            })?;

    for (passphrase, entropy) in passphrases {
        println!("{} <{}>", passphrase, entropy);
    }

    Ok(())
}
