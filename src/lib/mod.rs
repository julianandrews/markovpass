extern crate test;

mod alias_dist;
mod error;
mod markovchain;

use std;
use std::error::Error;
use std::path::PathBuf;

pub fn gen_passphrases(
    filename: Option<PathBuf>,
    number: usize,
    min_entropy: f64,
    ngram_length: usize,
    min_word_length: usize,
) -> Result<Vec<(String, f64)>, Box<Error>> {
    let corpus = try!(read_file_or_stdin(filename));
    let ngrams = get_ngrams(&corpus, ngram_length, min_word_length);
    let chain = try!(markovchain::PassphraseMarkovChain::new(ngrams));

    let mut passphrases = Vec::with_capacity(number);
    for _ in 0..number {
        passphrases.push(chain.passphrase(min_entropy));
    }

    Ok(passphrases)
}

fn get_ngrams(corpus: &str, ngram_length: usize, min_word_length: usize) -> Vec<String> {
    let corpus = corpus.to_lowercase();
    let words = corpus
        .split_whitespace()
        .filter_map(|word| clean_word(word, min_word_length));
    let cleaned_corpus = Some("")
        .into_iter()
        .chain(words)
        .collect::<Vec<&str>>()
        .join(" ");
    let count = cleaned_corpus.chars().count();
    let mut ngrams = Vec::with_capacity(count);
    let mut chars = cleaned_corpus.chars().cycle();
    for _ in 0..count {
        let ngram: String = chars.clone().take(ngram_length).collect();
        ngrams.push(ngram);
        chars.next();
    }

    ngrams
}

fn clean_word(word: &str, min_length: usize) -> Option<&str> {
    let word = word.trim_matches(|c| !is_word_char(c));

    if word.chars().all(is_word_char) && word.len() >= min_length {
        Some(word)
    } else {
        None
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || c == '\''
}

fn read_file_or_stdin(filename: Option<PathBuf>) -> Result<String, std::io::Error> {
    let mut input: Box<std::io::Read> = match filename {
        Some(filename) => Box::new(try!(std::fs::File::open(&filename))),
        None => Box::new(std::io::stdin()),
    };
    let mut data = String::new();
    try!(input.read_to_string(&mut data));

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            get_ngrams("this is a test", 3, 3),
            vec![
                " th", "thi", "his", "is ", "s t", " te", "tes", "est", "st ", "t t"
            ]
        );
        assert_eq!(
            get_ngrams("this is a test", 5, 3),
            vec![
                " this", "this ", "his t", "is te", "s tes", " test", "test ", "est t", "st th",
                "t thi",
            ]
        );
        assert_eq!(
            get_ngrams("this is a test", 3, 2),
            vec![
                " th", "thi", "his", "is ", "s i", " is", "is ", "s t", " te", "tes", "est", "st ",
                "t t",
            ]
        );
        assert_eq!(get_ngrams("this is a test", 3, 5).len(), 0);
        assert_eq!(
            get_ngrams("Some awes0me test", 6, 3),
            vec![
                " some ", "some t", "ome te", "me tes", "e test", " test ", "test s", "est so",
                "st som", "t some",
            ]
        );
        assert_eq!(
            get_ngrams("test'in", 3, 3),
            vec![" te", "tes", "est", "st'", "t'i", "'in", "in ", "n t"]
        );
    }

    #[test]
    fn test_gen_passphrases() {
        let p = get_testdata_pathbuf();
        let result = gen_passphrases(Some(p), 5, 80.0, 3, 5);
        assert!(result.is_ok(), "Passphrase generation failed.");
        let passphrases = result.unwrap();
        assert_eq!(passphrases.len(), 5);
    }

    #[bench]
    fn bench_get_ngrams(b: &mut test::Bencher) {
        let p = get_testdata_pathbuf();
        let corpus = read_file_or_stdin(Some(p)).unwrap();
        b.iter(|| get_ngrams(&corpus, 3, 5));
    }

    #[bench]
    fn bench_gen_passphrases(b: &mut test::Bencher) {
        let p = get_testdata_pathbuf();
        b.iter(|| gen_passphrases(Some(p.clone()), 1, 80.0, 3, 5));
    }

    fn get_testdata_pathbuf() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("testdata/Jane Austen - Pride and Prejudice.txt");

        p
    }
}
