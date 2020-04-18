extern crate rand;

use rand::Rng;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum AliasDistributionError {
    InvalidWeight,
    NullDistribution,
}

impl ::std::error::Error for AliasDistributionError {}

impl fmt::Display for AliasDistributionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AliasDistributionError::InvalidWeight => {
                write!(f, "Weights must be finite non-negative values.")
            }
            AliasDistributionError::NullDistribution => {
                write!(f, "Sum of weights must be non-zero.")
            }
        }
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
        if weights
            .iter()
            .any(|&w| w.is_sign_negative() || !w.is_finite())
        {
            return Err(AliasDistributionError::InvalidWeight);
        };
        let size = weights.len();
        let total: f64 = weights.iter().sum();
        if total == 0.0 {
            return Err(AliasDistributionError::NullDistribution);
        };
        let mut entropy = 0.0;

        let mut probability_table = Vec::with_capacity(size);
        for weight in weights {
            let prob = weight / total;
            entropy -= prob * prob.log(2.0);
            probability_table.push(prob * (size as f64));
        }

        let mut alias_table: Vec<usize> = (0..size).collect();
        let mut overfull: Vec<usize> = Vec::new();
        let mut underfull: Vec<usize> = Vec::new();
        for (i, &prob) in probability_table.iter().enumerate() {
            if prob < 1.0 {
                underfull.push(i);
            } else {
                overfull.push(i);
            }
        }
        while let (Some(i), Some(j)) = (underfull.pop(), overfull.pop()) {
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
        // TODO: Catch the potential error and let the caller decide what to do.
        let mut rng = rand::OsRng::new().unwrap();
        let i = rng.gen_range(0, self.probability_table.len());
        if self.probability_table[i] >= rng.gen() {
            i
        } else {
            self.alias_table[i]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {{
            let (a, b) = (&$a, &$b);
            assert!(
                (*a - *b).abs() < 1.0e-6,
                "{} is not approximately equal to {}",
                *a,
                *b
            );
        }};
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
        assert_eq!(err, AliasDistributionError::InvalidWeight);
    }

    #[test]
    fn test_nan_weight() {
        let err = AliasDistribution::new(&vec![3.2, f64::NAN, 4.5]).unwrap_err();
        assert_eq!(err, AliasDistributionError::InvalidWeight);
    }

    #[test]
    fn test_infinite() {
        let err = AliasDistribution::new(&vec![3.2, 0.5, f64::INFINITY]).unwrap_err();
        assert_eq!(err, AliasDistributionError::InvalidWeight);
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
