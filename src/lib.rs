mod alias_dist;

use alias_dist::AliasDistribution;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct MarkovNode<T: Hash + Eq + Clone> {
    pub value: T,
    dist: AliasDistribution<T>,
}

impl<T: Hash + Eq + Clone> MarkovNode<T> {
    pub fn new(value: T, counts: &HashMap<T, usize>) -> MarkovNode<T> {
        let dist = AliasDistribution::new(&counts);
        MarkovNode { value: value.clone(), dist: dist }
    }

    pub fn next(&self) -> &T {
        self.dist.choice()
    }

    pub fn entropy(&self) -> f64 {
        self.dist.entropy
    }
}

#[derive(Debug)]
pub struct PassphraseMarkovChain {
    nodes: HashMap<String, MarkovNode<String>>,
    starting_dist: AliasDistribution<String>,
}

impl PassphraseMarkovChain {
    pub fn new<U: Clone>(ngrams: U) -> Result<PassphraseMarkovChain, &'static str> where U: Iterator<Item=String> {
        let mut transition_map = HashMap::new();
        let mut starting_counts = HashMap::new();
        let mut ngrams_copy = ngrams.clone().cycle();
        ngrams_copy.next();
        for (a, b) in ngrams.zip(ngrams_copy) {
            if b.starts_with(" ") {
                let count = starting_counts.entry(b.clone()).or_insert(0);
                *count += 1
            }
            let transitions = transition_map.entry(a).or_insert(HashMap::new());
            let count = transitions.entry(b).or_insert(0);
            *count += 1;
        };

        let mut total_entropy: f64 = 0.0;
        let mut nodes = HashMap::new();
        for (ngram, transition_counts) in &transition_map {
            let node = MarkovNode::new(ngram.clone(), transition_counts);
            total_entropy += node.entropy();
            nodes.insert(ngram.clone(), node);
        };
        let starting_dist = AliasDistribution::new(&starting_counts);

        if total_entropy == 0.0 {
            return Err("No entropy found in input.");
        } else if starting_dist.entropy == 0.0 {
            return Err("No start of word entropy found in input.");
        };

        Ok(PassphraseMarkovChain {
            nodes: nodes,
            starting_dist: starting_dist,
        })
    }

    pub fn get_node(&self) -> &MarkovNode<String> {
        self.nodes.get(self.starting_dist.choice()).unwrap()
    }

    pub fn passphrase(&self, min_entropy: f64) -> (String, f64) {
        let mut node = self.get_node();
        let mut entropy = self.starting_dist.entropy;
        let mut nodes = Vec::new();
        loop {
            nodes.push(node);
            entropy += node.entropy();
            if entropy >= min_entropy && node.value.ends_with(" ") { break };
            node = self.nodes.get(node.next()).unwrap();
        }
        let tail = nodes.last().unwrap().value.chars().skip(1);
        let chars = nodes.iter().map(|n| n.value.chars().next().unwrap()).chain(tail);
        let passphrase = chars.collect::<String>().trim().to_string();
        (passphrase, entropy)
    }
}
