use std::collections::HashMap;
use lib::alias_dist::AliasDistribution;
use lib::errors::MarkovpassError;

#[derive(Debug)]
pub struct MarkovNode<T> {
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
    pub fn new<U: Iterator<Item = String>>(
        mut ngrams: U,
    ) -> Result<PassphraseMarkovChain, MarkovpassError> {
        // Count transitions and viable starting ngrams.
        // To get natural sounding words, starting ngrams should be at word start.
        let mut transition_counters: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut starting_ngram_counts: HashMap<String, usize> = HashMap::new();

        let first = ngrams.next();
        if first.is_none() {
            return Err(MarkovpassError::NoNgrams);
        }
        let mut a = first.unwrap();
        for b in ngrams.chain(Some(a.clone()).into_iter()) {
            let transitions = transition_counters.entry(a).or_insert(HashMap::new());
            let count = transitions.entry(b.clone()).or_insert(0);
            *count += 1;
            if b.starts_with(" ") {
                let count = starting_ngram_counts.entry(b.clone()).or_insert(0);
                *count += 1
            }
            a = b;
        }

        // Build all the MarkovNodes from the tranition counts.
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
            nodes.insert(ngram, node);
        }

        // Generate the starting ngram probability distribution.
        let mut starting_ngrams = Vec::with_capacity(starting_ngram_counts.len());
        let mut weights = Vec::with_capacity(starting_ngram_counts.len());
        for (value, weight) in starting_ngram_counts.into_iter() {
            starting_ngrams.push(value);
            weights.push(weight as f64);
        }
        let starting_dist = AliasDistribution::new(weights).unwrap();

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

    pub fn get_starting_ngram(&self) -> String {
        self.nodes
            .get(&self.starting_ngrams[self.starting_dist.choice()])
            .unwrap()
            .value
            .to_owned()
    }

    pub fn get_next_ngram(&self, ngram: &str) -> String {
        self.nodes.get(ngram).unwrap().next().to_owned()
    }

    pub fn get_ngram_entropy(&self, ngram: &str) -> f64 {
        self.nodes.get(ngram).unwrap().entropy()
    }

    pub fn passphrase(&self, min_entropy: f64) -> (String, f64) {
        let mut ngrams = Vec::new();

        let mut ngram = self.get_starting_ngram();
        let mut entropy = self.starting_dist.entropy;
        loop {
            ngrams.push(ngram.clone());
            entropy += self.get_ngram_entropy(&ngram);
            // The passphrase is complete when we have enough entropy, and hit a word end.
            if entropy >= min_entropy && ngram.ends_with(" ") {
                break;
            };
            ngram = self.get_next_ngram(&ngram);
        }
        // We want to include the first character from each node, but the whole ngram from the
        // final node.
        let tail = ngrams.last().unwrap().chars().skip(1);
        let chars = ngrams.iter().map(|n| n.chars().next().unwrap()).chain(tail);
        let passphrase = chars.collect::<String>().trim().to_string();

        (passphrase, entropy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markovnode_new() {
        let node = MarkovNode::new("tic", vec!["tac", "toc", "toe"], vec![1.0, 1.0, 2.0]);
        assert_eq!(node.entropy(), 1.5);
        assert_eq!(node.transitions.len(), 3);
        assert!(node.transitions.contains(node.next()));
    }

    #[test]
    fn test_passphrasemarkovchain_new() {
        let ngrams = vec![" ti", "tic", "ic ", "c t", " to", "toc", "oc ", "c t"];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.starting_ngrams.len(), 2);
        assert!(chain.starting_ngrams.contains(&" ti".to_string()));
        assert!(chain.starting_ngrams.contains(&" to".to_string()));
        assert_eq!(chain.starting_dist.entropy, 1.0);
        assert!(ngrams.contains(&chain.get_starting_ngram()));
        let (p, e) = chain.passphrase(60.0);
        assert_eq!(e, 60.0);
        assert_eq!(p.len(), 239);
    }

    #[test]
    fn test_passphrase_no_ngrams() {
        let ngrams: Vec<String> = vec![];
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovpassError::NoNgrams);
    }

    #[test]
    fn test_passphrase_no_entropy() {
        let ngrams = vec![" ab", "abc", "bcd", "cd ", "d a"];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovpassError::NoEntropy);
    }

    #[test]
    fn test_gen_passphrases_no_starting_entropy() {
        let ngrams = vec![
            " ab", "abc", "bc ", "c a", " ab", "abc", "cbd", "bd ", "d a"
        ];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MarkovpassError::NoStartOfWordEntropy);
    }
}
