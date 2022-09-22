use rand_distr::weighted_alias::WeightedAliasIndex;
use rand_distr::Distribution;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkovChainError {
    NoNgrams,
    ZeroEntropy,
    ZeroStartOfWordEntropy,
}

impl std::error::Error for MarkovChainError {}

impl fmt::Display for MarkovChainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NoNgrams => write!(f, "No ngrams found in cleaned input."),
            Self::ZeroEntropy => write!(f, "Cleaned input has no entropy."),
            Self::ZeroStartOfWordEntropy => {
                write!(f, "Cleaned input has no start of word entropy.")
            }
        }
    }
}

struct MarkovChainIterator<'chain> {
    markov_chain: &'chain PassphraseMarkovChain<'chain>,
    current: &'chain str,
}

impl<'chain> Iterator for MarkovChainIterator<'chain> {
    type Item = &'chain str;

    fn next(&mut self) -> Option<Self::Item> {
        let last = self.current;
        self.current = self.markov_chain.get_next_ngram(self.current);

        Some(last)
    }
}

#[derive(Debug)]
struct MarkovNode<T> {
    pub value: T,
    transitions: Vec<T>,
    dist: WeightedAliasIndex<f64>,
    entropy: f64,
}

impl<T> MarkovNode<T> {
    pub fn new(value: T, values: Vec<T>, weights: Vec<f64>) -> Self {
        let entropy = weight_entropy(&weights);
        Self {
            value,
            transitions: values,
            dist: WeightedAliasIndex::new(weights).unwrap(),
            entropy,
        }
    }

    pub fn next(&self) -> &T {
        &self.transitions[self.dist.sample(&mut rand::rngs::OsRng)]
    }

    pub const fn entropy(&self) -> f64 {
        self.entropy
    }
}

#[derive(Debug)]
pub struct PassphraseMarkovChain<'ngrams> {
    nodes: HashMap<&'ngrams str, MarkovNode<&'ngrams str>>,
    starting_ngrams: Vec<&'ngrams str>,
    starting_dist: WeightedAliasIndex<f64>,
    starting_entropy: f64,
}

impl<'ngrams> PassphraseMarkovChain<'ngrams> {
    pub fn new(
        ngrams: impl Iterator<Item = &'ngrams str>,
    ) -> Result<PassphraseMarkovChain<'ngrams>, MarkovChainError> {
        // Count transitions and viable starting ngrams.
        // To get natural sounding words, starting ngrams should be at word start.
        let mut transition_counters: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
        let mut starting_ngram_counts: HashMap<&str, usize> = HashMap::new();
        let mut ngrams = ngrams.peekable();
        let first_ngram = <&str>::clone(ngrams.peek().ok_or(MarkovChainError::NoNgrams)?);
        while let Some(current_ngram) = ngrams.next() {
            if current_ngram.starts_with(' ') {
                *starting_ngram_counts.entry(current_ngram).or_insert(0) += 1;
            }
            // To guarantee every ngram has at least one valid transition, let the last ngram
            // transition to the first.
            let next_ngram = ngrams.peek().unwrap_or(&first_ngram);
            *transition_counters
                .entry(current_ngram)
                .or_insert_with(HashMap::new)
                .entry(next_ngram)
                .or_insert(0) += 1;
        }

        // Generate the starting ngram probability distribution.
        let mut starting_ngrams = Vec::with_capacity(starting_ngram_counts.len());
        let mut starting_ngram_weights = Vec::with_capacity(starting_ngram_counts.len());
        for (value, weight) in starting_ngram_counts {
            starting_ngrams.push(value);
            starting_ngram_weights.push(weight as f64);
        }
        let starting_entropy = weight_entropy(&starting_ngram_weights);
        let starting_dist = WeightedAliasIndex::new(starting_ngram_weights).unwrap();

        // Build all the MarkovNodes from the transition counts.
        let mut nodes: HashMap<&str, MarkovNode<&str>> = HashMap::new();
        let mut total_entropy: f64 = 0.0;
        for (ngram, transition_counts) in transition_counters {
            let mut values = Vec::with_capacity(transition_counts.len());
            let mut weights = Vec::with_capacity(transition_counts.len());
            for (value, weight) in transition_counts {
                values.push(value);
                weights.push(weight as f64);
            }

            let node = MarkovNode::new(ngram, values, weights);
            total_entropy += node.entropy();
            nodes.insert(ngram, node);
        }

        if total_entropy == 0.0 {
            return Err(MarkovChainError::ZeroEntropy);
        }
        if starting_entropy == 0.0 {
            return Err(MarkovChainError::ZeroStartOfWordEntropy);
        }

        Ok(PassphraseMarkovChain {
            nodes,
            starting_ngrams,
            starting_dist,
            starting_entropy,
        })
    }

    pub fn passphrase(&self, min_entropy: f64) -> (String, f64) {
        let mut selected_ngrams = Vec::new();
        let mut entropy = self.starting_entropy;

        for ngram in self.iter() {
            selected_ngrams.push(ngram);
            entropy += self.ngram_entropy(ngram);
            if entropy >= min_entropy && ngram.ends_with(' ') {
                break;
            }
        }

        // Include the first character from each ngram, and the whole final ngram.
        let tail = selected_ngrams.last().unwrap().chars().skip(1);
        let chars = selected_ngrams
            .iter()
            .map(|n| n.chars().next().unwrap())
            .chain(tail);
        let passphrase = chars.collect::<String>().trim().to_string();

        (passphrase, entropy)
    }

    fn iter(&self) -> MarkovChainIterator {
        MarkovChainIterator {
            markov_chain: self,
            current: self.get_starting_ngram(),
        }
    }

    fn get_starting_ngram(&self) -> &str {
        self.nodes
            .get(&self.starting_ngrams[self.starting_dist.sample(&mut rand::rngs::OsRng)])
            .unwrap()
            .value
    }

    fn get_next_ngram(&self, ngram: &str) -> &str {
        self.nodes.get(ngram).unwrap().next()
    }

    fn ngram_entropy(&self, ngram: &str) -> f64 {
        self.nodes.get(ngram).unwrap().entropy()
    }
}

fn weight_entropy(weights: &[f64]) -> f64 {
    let total: f64 = weights.iter().sum();
    weights.iter().fold(0.0, |acc, weight| {
        let prob = weight / total;
        acc - prob * prob.log2()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passphrasemarkovchain_new() {
        let ngrams = vec![" ti", "tic", "ic ", "c t", " to", "toc", "oc ", "c t"];
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.starting_ngrams.len(), 2);
        assert!(chain.starting_ngrams.contains(&" ti"));
        assert!(chain.starting_ngrams.contains(&" to"));
        assert_eq!(chain.starting_entropy, 1.0);
        assert!(ngrams.contains(&chain.get_starting_ngram()));
        let (p, e) = chain.passphrase(60.0);
        assert_eq!(e, 60.0);
        assert_eq!(p.len(), 239);
    }

    #[test]
    fn test_passphrase_no_ngrams() {
        let result = PassphraseMarkovChain::new(std::iter::empty());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovChainError::NoNgrams);
    }

    #[test]
    fn test_passphrase_no_entropy() {
        let ngrams = vec![" ab", "abc", "bcd", "cd ", "d a"];
        let result = PassphraseMarkovChain::new(ngrams.into_iter());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovChainError::ZeroEntropy);
    }

    #[test]
    fn test_passphrases_no_starting_entropy() {
        let ngrams = vec![
            " ab", "abc", "bc ", "c a", " ab", "abc", "cbd", "bd ", "d a",
        ];
        let result = PassphraseMarkovChain::new(ngrams.into_iter());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            MarkovChainError::ZeroStartOfWordEntropy
        );
    }
}
