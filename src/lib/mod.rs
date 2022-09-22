#[cfg(feature = "benchmarks")]
extern crate test;

mod corpus;
mod markovchain;

use std::fs::File;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GenPassphraseOptions {
    pub filename: Option<PathBuf>,
    pub number: usize,
    pub min_entropy: f64,
    pub ngram_length: usize,
    pub min_word_length: usize,
}

pub fn gen_passphrases(
    options: &GenPassphraseOptions,
) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let reader: Box<dyn io::Read> = match &options.filename {
        Some(filename) => Box::new(io::BufReader::new(File::open(&filename)?)),
        None => Box::new(io::stdin()),
    };
    let corpus = corpus::Corpus::new(reader, options.ngram_length, options.min_word_length)?;
    let chain = markovchain::PassphraseMarkovChain::new(corpus.ngrams())?;

    let passphrases = (0..options.number)
        .map(|_| chain.passphrase(options.min_entropy))
        .collect();

    Ok(passphrases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_passphrases() {
        let result = gen_passphrases(&get_test_options());
        assert!(result.is_ok(), "Passphrase generation failed.");
        let passphrases = result.unwrap();
        assert_eq!(passphrases.len(), 5);
    }

    #[cfg(feature = "benchmarks")]
    #[bench]
    fn bench_gen_passphrases(b: &mut test::Bencher) {
        let options = get_test_options();
        b.iter(|| gen_passphrases(&options));
    }

    fn get_testdata_pathbuf() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("testdata/Jane Austen - Pride and Prejudice.txt");

        p
    }

    fn get_test_options() -> GenPassphraseOptions {
        GenPassphraseOptions {
            filename: Some(get_testdata_pathbuf()),
            number: 5,
            min_entropy: 80.0,
            ngram_length: 3,
            min_word_length: 5,
        }
    }
}

mod bench {}
