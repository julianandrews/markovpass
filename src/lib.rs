mod alias_dist;

use alias_dist::AliasDistribution;
use std::collections::HashMap;

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
            dist: AliasDistribution::new(weights),
        }
    }

    pub fn next(&self) -> &T {
        &self.transitions[self.dist.choice().unwrap()]
    }

    pub fn entropy(&self) -> f64 {
        self.dist.entropy
    }
}

#[derive(Debug)]
pub struct PassphraseMarkovChain {
    nodes: HashMap<String, MarkovNode<String>>,
    starting_nodes: Vec<String>,
    starting_dist: AliasDistribution,
}

impl PassphraseMarkovChain {
    pub fn new<U: Iterator<Item=String> + Clone>(ngrams: U)
            -> Result<PassphraseMarkovChain, &'static str> {
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
        for (ngram, transition_counts) in transition_map.into_iter() {

            // FIXME: Temporary
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

        // FIXME: Temporary
        let mut starting_nodes = Vec::with_capacity(starting_counts.len());
        let mut weights = Vec::with_capacity(starting_counts.len());
        for (value, weight) in starting_counts.into_iter() {
            starting_nodes.push(value);
            weights.push(weight as f64);
        }
        let starting_dist = AliasDistribution::new(weights);

        if total_entropy == 0.0 {
            return Err("No entropy found in input.");
        } else if starting_dist.entropy == 0.0 {
            return Err("No start of word entropy found in input.");
        };

        Ok(PassphraseMarkovChain {
            nodes: nodes,
            starting_nodes: starting_nodes,
            starting_dist: starting_dist,
        })
    }

    pub fn get_node(&self) -> &MarkovNode<String> {
        self.nodes.get(&self.starting_nodes[self.starting_dist.choice().unwrap()]).unwrap()
    }

    pub fn passphrase(&self, min_entropy: f64) -> (String, f64) {
        let mut node = self.get_node();
        let mut entropy = self.starting_dist.entropy;
        let mut nodes = Vec::new();
        loop {
            nodes.push(node);
            entropy += node.entropy();
            if entropy >= min_entropy && node.value.ends_with(" ") { break };
            let ngram = node.next();
            node = self.nodes.get(ngram).unwrap();
        }
        let tail = nodes.last().unwrap().value.chars().skip(1);
        let chars = nodes.iter().map(|n| n.value.chars().next().unwrap()).chain(tail);
        let passphrase = chars.collect::<String>().trim().to_string();

        (passphrase, entropy)
    }
}
