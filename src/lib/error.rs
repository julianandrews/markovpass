use std;

#[derive(Debug, PartialEq)]
pub enum MarkovpassError {
    NoNgrams,
    NoEntropy,
    NoStartOfWordEntropy,
}

impl std::error::Error for MarkovpassError {}

impl std::fmt::Display for MarkovpassError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            MarkovpassError::NoNgrams => write!(f, "No ngrams found in cleaned input."),
            MarkovpassError::NoEntropy => write!(f, "Cleaned input has no entropy."),
            MarkovpassError::NoStartOfWordEntropy => {
                write!(f, "Cleaned input has not start of word entropy.")
            }
        }
    }
}
