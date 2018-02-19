use std;

#[derive(Debug, PartialEq)]
pub enum MarkovpassError {
    NoNgrams,
    NoEntropy,
    NoStartOfWordEntropy,
}

impl std::fmt::Display for MarkovpassError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            MarkovpassError::NoNgrams => write!(f, "NoNgrams"),
            MarkovpassError::NoEntropy => write!(f, "NoEntropy"),
            MarkovpassError::NoStartOfWordEntropy => write!(f, "NoStartOfWordEntropy"),
        }
    }
}

impl std::error::Error for MarkovpassError {
    fn description(&self) -> &str {
        match *self {
            MarkovpassError::NoNgrams => "No ngrams found in input",
            MarkovpassError::NoEntropy => "Input has no entropy",
            MarkovpassError::NoStartOfWordEntropy => "Input has no start of word entropy",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}
