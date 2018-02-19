extern crate test;

mod alias_dist;
mod errors;
mod markovchain;

use std;

pub fn gen_passphrases<U: Iterator<Item = String>>(
    ngrams: U,
    number: usize,
    min_entropy: f64,
) -> Result<Vec<(String, f64)>, errors::MarkovpassError> {
    let chain = try!(markovchain::PassphraseMarkovChain::new(ngrams));
    let mut passphrases = Vec::with_capacity(number);
    for _ in 0..number {
        passphrases.push(chain.passphrase(min_entropy));
    }

    Ok(passphrases)
}

pub fn get_ngrams(
    filename: Option<&str>,
    ngram_length: usize,
    min_word_length: usize,
) -> Result<std::vec::IntoIter<String>, std::io::Error> {
    let corpus = try!(get_corpus(filename));
    Ok(get_corpus_ngrams(corpus, ngram_length, min_word_length))
}

fn get_corpus(filename: Option<&str>) -> Result<String, std::io::Error> {
    let mut input: Box<std::io::Read> = match filename {
        Some(filename) => Box::new(try!(std::fs::File::open(&filename))),
        None => Box::new(std::io::stdin()),
    };
    let mut corpus = String::new();
    try!(input.read_to_string(&mut corpus));

    Ok(corpus)
}

fn get_corpus_ngrams(
    corpus: String,
    ngram_length: usize,
    min_word_length: usize,
) -> std::vec::IntoIter<String> {
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

    ngrams.into_iter()
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || c == '\''
}

fn clean_word(word: &str, min_length: usize) -> Option<&str> {
    let word = word.trim_matches(|c| !is_word_char(c));

    if word.chars().all(is_word_char) && word.len() >= min_length {
        Some(word)
    } else {
        None
    }
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
    fn test_get_corpus_ngrams() {
        assert_eq!(
            get_corpus_ngrams("this is a test".to_string(), 3, 3).collect::<Vec<String>>(),
            vec![
                " th", "thi", "his", "is ", "s t", " te", "tes", "est", "st ", "t t"
            ]
        );
        assert_eq!(
            get_corpus_ngrams("this is a test".to_string(), 5, 3).collect::<Vec<String>>(),
            vec![
                " this", "this ", "his t", "is te", "s tes", " test", "test ", "est t", "st th",
                "t thi",
            ]
        );
        assert_eq!(
            get_corpus_ngrams("this is a test".to_string(), 3, 2).collect::<Vec<String>>(),
            vec![
                " th", "thi", "his", "is ", "s i", " is", "is ", "s t", " te", "tes", "est", "st ",
                "t t",
            ]
        );
        assert_eq!(
            get_corpus_ngrams("this is a test".to_string(), 3, 5).next(),
            None
        );
        assert_eq!(
            get_corpus_ngrams("Some awes0me test".to_string(), 6, 3).collect::<Vec<String>>(),
            vec![
                " some ", "some t", "ome te", "me tes", "e test", " test ", "test s", "est so",
                "st som", "t some",
            ]
        );
        assert_eq!(
            get_corpus_ngrams("test'in".to_string(), 3, 3).collect::<Vec<String>>(),
            vec![" te", "tes", "est", "st'", "t'i", "'in", "in ", "n t"]
        );
    }

    #[bench]
    fn bench_get_corpus_ngrams(b: &mut test::Bencher) {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("testdata/Jane Austen - Pride and Prejudice.txt");
        let corpus = get_corpus(p.to_str()).unwrap();
        b.iter(|| get_corpus_ngrams(corpus.clone(), 3, 5));
    }
}
