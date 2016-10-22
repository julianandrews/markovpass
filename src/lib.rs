mod alias_dist;

use alias_dist::AliasDistribution;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ToNgrams<'a> {
    string: &'a str,
    ngram_start: usize,
    ngram_end: usize,
}

impl<'a> Iterator for ToNgrams<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        // check if ngram_start is past the end of the str, if so return None
        // build the slice, wrapping if ngram_end < ngram_start
        // increment ngram_start
        // increment ngram_end, wrapping if necessary

        // http://stackoverflow.com/questions/29670170/iterate-over-a-string-n-elements-at-a-time
        // unsafe {
        //     if let Some(next_end_char) = self.string[self.ngram_end..].chars().next() {
        //         let ngram = self.string.slice_unchecked(self.ngram_start, self.ngram_end);
        //         self.ngram_end += next_end_char.len_utf8();
        //         let next_start_char = self.string[self.ngram_start..].chars().next().unwrap();
        //         self.ngram_start += next_start_char.len_utf8();

        //         Some(ngram)
        //     } else {
        //         None
        //     }
        // }
    }
}

impl<'a> ToNgrams<'a> {
    pub fn new(ngram_length: usize, mut string: &'a str) -> ToNgrams<'a> {
        // let extra_chars = if let Some(x) = string.char_indices().nth(ngram_length) {
        //     string[0..x.0].to_owned()
        // } else {
        //     " ".to_string()
        // };
        // let last_char_length = extra_chars.chars().next_back().unwrap().len_utf8();
        // string.push_str(&extra_chars);
        let ngram_end = string.char_indices().nth(ngram_length - 1).unwrap().0; 
        ToNgrams {
            string: string,
            ngram_start: 0,
            ngram_end: ngram_end,
        }
    }
}

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

#[derive(Debug, PartialEq)]
pub enum PassphraseMarkovChainError {
    NoNgrams,
    NoEntropy,
    NoStartOfWordEntropy,
}

impl std::fmt::Display for PassphraseMarkovChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match * self {
            PassphraseMarkovChainError::NoNgrams => write!(f, "NoNgrams`"),
            PassphraseMarkovChainError::NoEntropy => write!(f, "NoEntropy"),
            PassphraseMarkovChainError::NoStartOfWordEntropy => write!(f, "NoStartOfWordEntropy"),
        }
    }
}

impl std::error::Error for PassphraseMarkovChainError {
    fn description(&self) -> &str {
        match *self {
            PassphraseMarkovChainError::NoNgrams => "No ngrams found in input",
            PassphraseMarkovChainError::NoEntropy => "Input has no entropy",
            PassphraseMarkovChainError::NoStartOfWordEntropy =>
                "Input has no start of word entropy",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

#[derive(Debug)]
pub struct PassphraseMarkovChain {
    nodes: HashMap<String, MarkovNode<String>>,
    starting_ngrams: Vec<String>,
    starting_dist: AliasDistribution,
}

impl PassphraseMarkovChain {
    pub fn new<U: Iterator<Item=String> + Clone>(ngrams: U)
            -> Result<PassphraseMarkovChain, PassphraseMarkovChainError> {
        // Count transitions and viable starting ngrams.
        // To get natural sounding words, only start with ngrams at the start of words.
        let mut transition_counters: HashMap<String, HashMap<String, usize>> = HashMap::new();
        let mut starting_ngram_counts: HashMap<String, usize> = HashMap::new();
        let mut ngrams_copy = ngrams.clone().cycle();
        if ngrams_copy.next().is_none() {
            return Err(PassphraseMarkovChainError::NoNgrams);
        }
        for (a, b) in ngrams.zip(ngrams_copy).into_iter() {
            if b.starts_with(" ") {
                let count = starting_ngram_counts.entry(b.clone()).or_insert(0);
                *count += 1
            }
            let transitions = transition_counters.entry(a).or_insert(HashMap::new());
            let count = transitions.entry(b).or_insert(0);
            *count += 1;
        };

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
        };

        // Generate the starting ngram probability distribution.
        let mut starting_ngrams = Vec::with_capacity(starting_ngram_counts.len());
        let mut weights = Vec::with_capacity(starting_ngram_counts.len());
        for (value, weight) in starting_ngram_counts.into_iter() {
            starting_ngrams.push(value);
            weights.push(weight as f64);
        }
        let starting_dist = AliasDistribution::new(weights).unwrap();

        if total_entropy == 0.0 {
            return Err(PassphraseMarkovChainError::NoEntropy);
        } else if starting_dist.entropy == 0.0 {
            return Err(PassphraseMarkovChainError::NoStartOfWordEntropy);
        };

        Ok(PassphraseMarkovChain {
            nodes: nodes,
            starting_ngrams: starting_ngrams,
            starting_dist: starting_dist,
        })
    }

    pub fn get_node(&self) -> &MarkovNode<String> {
        self.nodes.get(&self.starting_ngrams[self.starting_dist.choice()]).unwrap()
    }

    pub fn passphrase(&self, min_entropy: f64) -> (String, f64) {
        let mut node = self.get_node();
        let mut entropy = self.starting_dist.entropy;
        let mut nodes = Vec::new();
        loop {
            nodes.push(node);
            entropy += node.entropy();
            // The passphrase is complete when we have enough entropy, and hit a word end.
            if entropy >= min_entropy && node.value.ends_with(" ") { break };
            let ngram = node.next();
            node = self.nodes.get(ngram).unwrap();
        }
        // We want to include the first character from each node, but the whole ngram from the
        // final node.
        let tail = nodes.last().unwrap().value.chars().skip(1);
        let chars = nodes.iter().map(|n| n.value.chars().next().unwrap()).chain(tail);
        let passphrase = chars.collect::<String>().trim().to_string();

        (passphrase, entropy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markovnode_new() {
        let node = MarkovNode::new("tic", vec! ["tac", "toc", "toe"], vec! [1.0, 1.0, 2.0]);
        assert_eq!(node.entropy(), 1.5);
        assert_eq!(node.transitions.len(), 3);
        assert!(node.transitions.contains(node.next()));
    }

    #[test]
    fn test_passphrasemarkovchain_new() {
        let ngrams = vec! [" ti", "tic", "ic ", "c t", " to", "toc", "oc ", "c t"];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.starting_ngrams.len(), 2);
        assert!(chain.starting_ngrams.contains(&" ti".to_string()));
        assert!(chain.starting_ngrams.contains(&" to".to_string()));
        assert_eq!(chain.starting_dist.entropy, 1.0);
        assert!(ngrams.contains(&chain.get_node().value));
        let (p, e) = chain.passphrase(60.0);
        assert_eq!(e, 60.0);
        assert_eq!(p.len(), 239);
    }

    #[test]
    fn test_passphrase_no_ngrams() {
        let ngrams: Vec<String> = vec![];
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PassphraseMarkovChainError::NoNgrams);
    }

    #[test]
    fn test_passphrase_no_entropy() {
        let ngrams = vec! [" ab", "abc", "bcd", "cd ", "d a"];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PassphraseMarkovChainError::NoEntropy);
    }

    #[test]
    fn test_gen_passphrases_no_starting_entropy() {
        let ngrams = vec! [" ab", "abc", "bc ", "c a", " ab", "abc", "cbd", "bd ", "d a"];
        let ngrams: Vec<String> = ngrams.iter().map(|x| x.to_string()).collect();
        let result = PassphraseMarkovChain::new(ngrams.iter().cloned());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PassphraseMarkovChainError::NoStartOfWordEntropy);
    }
}
