mod alias_dist;

use alias_dist::AliasDistribution;
use std::collections::HashMap;
use std::hash::Hash;

pub struct MarkovNode<T: Hash + Eq + Clone> {
    pub value: T,
    dist: AliasDistribution<T>,
}

impl<T: Hash + Eq + Clone> MarkovNode<T> {
    pub fn new(value: T, counts: &HashMap<T, usize>) -> MarkovNode<T> {
        let dist = AliasDistribution::new(&counts);
        MarkovNode { value: value.clone(), dist: dist }
    }

    pub fn get_transition(&self) -> &T {
        self.dist.choice()
    }
}

pub fn build_markov_chain<T: Hash + Eq + Clone, U: Clone>(
    ngrams: U
    ) -> HashMap<T, MarkovNode<T>> where U: Iterator<Item=T> {
    let mut transition_map = HashMap::new();
    let mut ngrams_copy = ngrams.clone().cycle();
    ngrams_copy.next();
    for (a, b) in ngrams.zip(ngrams_copy) {
        let transitions = transition_map.entry(a).or_insert(HashMap::new());
        let count = transitions.entry(b).or_insert(0);
        *count += 1;
    };
    let mut chain = HashMap::new();
    for (ngram, transition_counts) in &transition_map {
        let node = MarkovNode::new(ngram.clone(), transition_counts);
        chain.insert(ngram.clone(), node);
    };
    chain
}
