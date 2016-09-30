extern crate rand;

use self::rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct AliasDistribution<T: Hash + Eq> {
    values: Vec<T>,
    probability_table: Vec<f64>,
    alias_table: Vec<usize>,
    pub entropy: f64,
}

impl<T: Hash + Eq> AliasDistribution<T> {
    pub fn new(weights: HashMap<T, usize>) -> AliasDistribution<T> {
        let size = weights.len();
        let total = weights.values().fold(0, |sum, x| sum + x) as f64;
        let mut entropy = 0.0;

        let mut values = Vec::with_capacity(size);
        let mut probability_table = Vec::with_capacity(size);
        for (value, weight) in weights.into_iter() {
            let prob = (weight as f64) / total;
            entropy -= prob * prob.log(2.0);
            values.push(value);
            probability_table.push(prob * (size as f64));
        };

        let mut alias_table: Vec<usize> = (0..size).collect();
        let mut overfull: Vec<usize> = Vec::with_capacity(size);
        let mut underfull: Vec<usize> = Vec::with_capacity(size);
        for (i, prob) in probability_table.iter().enumerate() {
            if *prob < 1.0 { underfull.push(i); } else { overfull.push(i); }
        };
        loop {
            if overfull.is_empty() || underfull.is_empty() { break; };
            let i = underfull.pop().unwrap();
            let j = overfull.pop().unwrap();
            alias_table[i] = j;
            probability_table[j] += probability_table[i] - 1.0;
            if probability_table[j] < 1.0 { underfull.push(j) } else { overfull.push(j) };
        };

        AliasDistribution {
            values: values,
            probability_table: probability_table,
            alias_table: alias_table,
            entropy: entropy,
        }
    }

    pub fn choice(&self) -> Option<&T> {
        if self.values.len() == 0 { return None; };
        let mut rng = rand::OsRng::new().unwrap();
        let i = rng.gen_range(0, self.values.len());
        let y = rng.gen();
        let choice = if self.probability_table[i] >= y { i } else { self.alias_table[i] };

        Some(&self.values[choice])
    }
}
