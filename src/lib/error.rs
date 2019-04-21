use std::fmt;

#[derive(Debug, PartialEq)]
pub enum MarkovpassError {
    NoNgrams,
    NoEntropy,
    NoStartOfWordEntropy,
}

impl ::std::error::Error for MarkovpassError {}

impl fmt::Display for MarkovpassError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MarkovpassError::NoNgrams => write!(f, "No ngrams found in cleaned input."),
            MarkovpassError::NoEntropy => write!(f, "Cleaned input has no entropy."),
            MarkovpassError::NoStartOfWordEntropy => {
                write!(f, "Cleaned input has not start of word entropy.")
            }
        }
    }
}
