extern crate rand;

use self::rand::Rng;

#[derive(Debug, PartialEq)]
pub enum AliasDistributionError {
    InvalidWeights,
    NullDistribution,
}

impl ::std::fmt::Display for AliasDistributionError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            AliasDistributionError::InvalidWeights => write!(f, "InvalidWeights"),
            AliasDistributionError::NullDistribution => write!(f, "NullDistribution"),
        }
    }
}

impl ::std::error::Error for AliasDistributionError {
    fn description(&self) -> &str {
        match *self {
            AliasDistributionError::InvalidWeights => "Negative weights are not allowed",
            AliasDistributionError::NullDistribution => "Sum of weights must not be zero",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        None
    }
}

#[derive(Debug)]
pub struct AliasDistribution {
    probability_table: Vec<f64>,
    alias_table: Vec<usize>,
    pub entropy: f64,
}

impl AliasDistribution {
    pub fn new(weights: &Vec<f64>) -> Result<AliasDistribution, AliasDistributionError> {
        if weights.iter().any(|&w| w < 0.0) {
            return Err(AliasDistributionError::InvalidWeights);
        };
        let size = weights.len();
        let total = weights.iter().fold(0.0, |sum, x| sum + x);
        if total == 0.0 {
            return Err(AliasDistributionError::NullDistribution);
        };
        let mut entropy = 0.0;

        let mut probability_table = Vec::with_capacity(size);
        for weight in weights.iter() {
            let prob = weight / total;
            entropy -= prob * prob.log(2.0);
            probability_table.push(prob * (size as f64));
        }

        let mut alias_table: Vec<usize> = (0..size).collect();
        let mut overfull: Vec<usize> = Vec::new();
        let mut underfull: Vec<usize> = Vec::new();
        for (i, prob) in probability_table.iter().enumerate() {
            if *prob < 1.0 {
                underfull.push(i);
            } else {
                overfull.push(i);
            }
        }
        loop {
            if overfull.is_empty() || underfull.is_empty() {
                break;
            };
            let i = underfull.pop().unwrap();
            let j = overfull.pop().unwrap();
            alias_table[i] = j;
            probability_table[j] += probability_table[i] - 1.0;
            if probability_table[j] < 1.0 {
                underfull.push(j)
            } else {
                overfull.push(j)
            };
        }

        Ok(AliasDistribution {
            probability_table: probability_table,
            alias_table: alias_table,
            entropy: entropy,
        })
    }

    pub fn choice(&self) -> usize {
        let mut rng = rand::OsRng::new().unwrap();
        let i = rng.gen_range(0, self.probability_table.len());
        let y = rng.gen();
        let choice = if self.probability_table[i] >= y {
            i
        } else {
            self.alias_table[i]
        };

        choice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => ({
            let (a, b) = (&$a, &$b);
            assert!((*a - *b).abs() < 1.0e-6, "{} is not approximately equal to {}", *a, *b);
        })
    }

    #[test]
    fn test_new() {
        let mut test_cases = Vec::new();
        test_cases.push(vec![1.0, 2.0, 3.0, 4.0]);
        test_cases.push(vec![0.2, 0.2, 0.2, 0.2, 0.2]);
        test_cases.push(vec![3.0, 0.0, 5.0]);
        test_cases.push(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);

        for weights in test_cases {
            let total = weights.iter().fold(0.0, |x, sum| sum + x);
            let count = weights.len() as f64;
            let dist = AliasDistribution::new(&weights).unwrap();
            let mut ps = vec![0.0; weights.len()];
            for (i, (&j, p)) in dist
                .alias_table
                .iter()
                .zip(dist.probability_table)
                .enumerate()
            {
                ps[i] += p / count;
                ps[j] += (1.0 - p) / count;
            }
            for (p, w) in ps.iter().zip(weights) {
                assert_approx_eq!(p, w / total);
            }
        }
    }

    #[test]
    fn test_negative_weight() {
        let err = AliasDistribution::new(&vec![3.2, -0.3, 4.5]).unwrap_err();
        assert_eq!(err, AliasDistributionError::InvalidWeights);
    }

    #[test]
    fn test_zero_distribution() {
        let err = AliasDistribution::new(&vec![0.0, 0.0, 0.0]).unwrap_err();
        assert_eq!(err, AliasDistributionError::NullDistribution);
    }

    #[test]
    fn test_empty_distribution() {
        let err = AliasDistribution::new(&Vec::new()).unwrap_err();
        assert_eq!(err, AliasDistributionError::NullDistribution);
    }

    #[test]
    fn test_entropy() {
        let dist = AliasDistribution::new(&vec![1.0, 1.0]).unwrap();
        assert_approx_eq!(dist.entropy, 1.0);
        let dist = AliasDistribution::new(&vec![0.5, 0.5, 1.0, 2.0]).unwrap();
        assert_approx_eq!(dist.entropy, 1.75);
    }

    #[test]
    fn test_choice() {
        let dist = AliasDistribution::new(&vec![1.0, 2.0, 3.0]).unwrap();
        let choice = dist.choice();
        assert!(choice < 3, "{} is too large");
    }
}
