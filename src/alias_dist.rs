extern crate rand;

use self::rand::Rng;

#[derive(Debug)]
pub struct AliasDistribution {
    probability_table: Vec<f64>,
    alias_table: Vec<usize>,
    pub entropy: f64,
}

impl AliasDistribution {
    pub fn new(weights: Vec<f64>) -> AliasDistribution {
        let size = weights.len();
        let total = weights.iter().fold(0.0, |sum, x| sum + x);
        let mut entropy = 0.0;

        let mut probability_table = Vec::with_capacity(size);
        for weight in weights.into_iter() {
            let prob = weight / total;
            entropy -= prob * prob.log(2.0);
            probability_table.push(prob * (size as f64));
        };

        let mut alias_table: Vec<usize> = (0..size).collect();
        let mut overfull: Vec<usize> = Vec::new();
        let mut underfull: Vec<usize> = Vec::new();
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
            probability_table: probability_table,
            alias_table: alias_table,
            entropy: entropy,
        }
    }

    pub fn choice(&self) -> Option<usize> {
        if self.probability_table.is_empty() { return None; };
        let mut rng = rand::OsRng::new().unwrap();
        let i = rng.gen_range(0, self.probability_table.len());
        let y = rng.gen();
        let choice = if self.probability_table[i] >= y { i } else { self.alias_table[i] };

        Some(choice)
    }
}
