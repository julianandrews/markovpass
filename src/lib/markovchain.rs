use std::collections::HashMap;
use lib::alias_dist::AliasDistribution;
use lib::error::MarkovpassError;

struct MarkovChainIterator<'a> {
    markov_chain: &'a PassphraseMarkovChain,
    current: &'a str,
}

impl<'a> Iterator for MarkovChainIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let last = self.current;
        self.current = self.markov_chain.get_next_ngram(&self.current);

        Some(last)
    }
}

#[derive(Debug)]
struct MarkovNode<T> {
    pub value: T,
    transitions: Vec<T>,
    dist: AliasDistribution,
}

impl<T> MarkovNode<T> {
    pub fn new(value: T, values: Vec<T>, weights: Vec<f64>) -> MarkovNode<T> {
        MarkovNode {
            value: value,
            transitions: values,
            dist: AliasDistribution::new(weights).unwrap(),
        }
    }

    pub fn next(&self) -> &T {
        &self.transitions[self.dist.choice()]
    }

    pub fn entropy(&self) -> f64 {
        self.dist.entropy
    }
}

#[derive(Debug)]
pub struct PassphraseMarkovChain {
    nodes: HashMap<String, MarkovNode<String>>,
    starting_ngrams: Vec<String>,
    starting_dist: AliasDistribution,
}

impl PassphraseMarkovChain {
    pub fn new(ngrams: Vec<String>) -> Result<PassphraseMarkovChain, MarkovpassError> {
        // Count transitions and viable starting ngrams.
        // To get natural sounding words, starting ngrams should be at word start.
        let mut transition_counters: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut starting_ngram_counts: HashMap<String, usize> = HashMap::new();

        if ngrams.len() == 0 {
            return Err(MarkovpassError::NoNgrams);
        }
        let mut a = ngrams[0].clone();
        for b in ngrams
            .into_iter()
            .skip(1)
            .chain(Some(a.clone()).into_iter())
        {
            if a.starts_with(" ") {
                *starting_ngram_counts.entry(a.clone()).or_insert(0) += 1;
            }
            *transition_counters
                .entry(a)
                .or_insert(HashMap::new())
                .entry(b.clone())
                .or_insert(0) += 1;
            a = b;
        }

        // Generate the starting ngram probability distribution.
        let mut starting_ngrams = Vec::with_capacity(starting_ngram_counts.len());
        let mut weights = Vec::with_capacity(starting_ngram_counts.len());
        for (value, weight) in starting_ngram_counts.into_iter() {
            starting_ngrams.push(value);
            weights.push(weight as f64);
        }
        let starting_dist = AliasDistribution::new(weights).unwrap();

        // Build all the MarkovNodes from the transition counts.
        let mut nodes: HashMap<String, MarkovNode<String>> = HashMap::new();
        let mut total_entropy: f64 = 0.0;
        for (ngram, transition_counts) in transition_counters.into_iter() {
            let mut values = Vec::with_capacity(transition_counts.len());
            let mut weights = Vec::with_capacity(transition_counts.len());
            for (value, weight) in transition_counts.into_iter() {
                values.push(value.clone());
                weights.push(weight as f64);
            }

            let node = MarkovNode::new(ngram.clone(), values, weights);
            total_entropy += node.entropy();
            nodes.insert(ngram.to_string(), node);
        }

        if total_entropy == 0.0 {
            return Err(MarkovpassError::NoEntropy);
        } else if starting_dist.entropy == 0.0 {
            return Err(MarkovpassError::NoStartOfWordEntropy);
        };

        Ok(PassphraseMarkovChain {
            nodes: nodes,
            starting_ngrams: starting_ngrams,
            starting_dist: starting_dist,
        })
    }

    pub fn passphrase(&self, min_entropy: f64) -> (String, f64) {
        let mut selected_ngrams = Vec::new();
        let mut entropy = self.starting_dist.entropy;

        for ngram in self.iter() {
            selected_ngrams.push(ngram.clone());
            entropy += self.ngram_entropy(&ngram);
            if entropy >= min_entropy && ngram.ends_with(" ") {
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
            markov_chain: &self,
            current: &self.get_starting_ngram(),
        }
    }

    fn get_starting_ngram(&self) -> &str {
        &self.nodes
            .get(&self.starting_ngrams[self.starting_dist.choice()])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passphrasemarkovchain_new() {
        let ngrams = vec![" ti", "tic", "ic ", "c t", " to", "toc", "oc ", "c t"];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.clone());
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.starting_ngrams.len(), 2);
        assert!(chain.starting_ngrams.contains(&" ti".to_string()));
        assert!(chain.starting_ngrams.contains(&" to".to_string()));
        assert_eq!(chain.starting_dist.entropy, 1.0);
        assert!(ngrams.contains(&chain.get_starting_ngram().to_string()));
        let (p, e) = chain.passphrase(60.0);
        assert_eq!(e, 60.0);
        assert_eq!(p.len(), 239);
    }

    #[test]
    fn test_passphrase_no_ngrams() {
        let ngrams: Vec<String> = vec![];
        let result = PassphraseMarkovChain::new(ngrams.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovpassError::NoNgrams);
    }

    #[test]
    fn test_passphrase_no_entropy() {
        let ngrams = vec![" ab", "abc", "bcd", "cd ", "d a"];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovpassError::NoEntropy);
    }

    #[test]
    fn test_passphrases_no_starting_entropy() {
        let ngrams = vec![
            " ab", "abc", "bc ", "c a", " ab", "abc", "cbd", "bd ", "d a"
        ];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovpassError::NoStartOfWordEntropy);
    }
}
