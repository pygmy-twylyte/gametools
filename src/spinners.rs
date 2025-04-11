//! Spinners Module
//!
//! Implements a game Spinner, comprised of Wedges which can be uniform or of different
//! relative widths, and can be blocked / covered according to game conditions. Wedges
//! may contain numeric values, strings, enums, or other user-defined types.
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Wedge<T>
where
    T: Clone,
{
    pub value: T,
    pub width: usize,
    pub active: bool,
}
impl<T: Clone> Wedge<T> {
    /// Create a new wedge to place on the spinner.
    pub fn new(value: T, width: usize, active: bool) -> Self {
        Self {
            value,
            width,
            active,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spinner<T>
where
    T: Clone,
{
    wedges: Vec<Wedge<T>>,
    weights: Vec<usize>,
}

impl<T: Clone> Spinner<T> {
    /// Create a new spinner with a vector of wedges.
    pub fn new(wedges: Vec<Wedge<T>>) -> Self {
        let weights = wedges.iter().map(|w| w.width).collect();
        Self { wedges, weights }
    }

    /// Spins the spinner, returning Some(value) of the wedge it lands on.
    /// Returns `None` if there are no wedges, or if the wedge selected is inactive / covered.
    /// The probability of landing on a particular wedge is determine by its width.
    /// 
    /// ## Example
    /// ```
    /// use gametools::spinners::{Spinner, Wedge};
    /// let loaded_coin = Spinner::new(vec![
    ///     Wedge::new("Heads", 75, true),
    ///     Wedge::new("Tails", 25, true),
    /// ]);
    /// let toss = loaded_coin.spin().unwrap();  // will be "Heads" 75% of the time
    /// ```
    pub fn spin(&self) -> Option<T> {
        if self.wedges.is_empty() {
            return None;
        }
        let mut rng = rand::rng();
        let distribution = WeightedIndex::new(&self.weights).ok()?;
        let chosen_wedge = self.wedges[distribution.sample(&mut rng)].clone();
        if !chosen_wedge.active {
            return None;
        }
        Some(self.wedges[distribution.sample(&mut rng)].clone().value)
    }
}

#[cfg(test)]
mod spinner_tests {
    use crate::spinners::*;
    #[test]
    fn can_create_wedges_with_varied_types() {
        let text_wedge = Wedge::new("Winner".to_string(), 1, true);
        assert_eq!(text_wedge.value, "Winner");
        let numeric = Wedge::new(10, 1, true);
        assert_eq!(numeric.value, 10);
    }

    #[test]
    fn can_create_spinners_with_varied_wedge_types() {
        let num_wedges = vec![
            Wedge::new(100, 1, true),
            Wedge::new(200, 1, true),
            Wedge::new(500, 1, true),
        ];
        let numeric_spinner = Spinner::new(num_wedges);
        assert_eq!(numeric_spinner.wedges.len(), 3);

        let text_wedges = vec![
            Wedge::new("Lose a Turn".to_string(), 2, true),
            Wedge::new("Ahead 4".to_string(), 4, true),
            Wedge::new("Back 2".to_string(), 4, true),
        ];
        let text_spinner = Spinner::new(text_wedges);
        assert_eq!(text_spinner.wedges.len(), 3);
        dbg!(text_spinner);
    }

    #[test]
    fn spin_returns_none_if_no_wedges_in_place() {
        let wedges: Vec<Wedge<usize>> = Vec::new();
        let spinner = Spinner::new(wedges);
        assert!(spinner.spin().is_none());
    }

    #[test]
    fn spin_always_returns_some_if_wedges_in_place() {
        let spinner = Spinner::new(vec![
            Wedge::new("Heads", 1, true),
            Wedge::new("Tails", 1, true),
        ]);
        for _ in 1..100 {
            assert!(spinner.spin().is_some());
        }
    }

    #[test]
    fn spin_returns_only_expected_values() {
        let spinner = Spinner::new(vec![
            Wedge::new(1, 1, true),
            Wedge::new(2, 1, true),
            Wedge::new(3, 1, true),
        ]);
        for _ in 1..1000 {
            assert!((1..=3).contains(&spinner.spin().unwrap()));
        }
    }

    #[test]
    fn spin_respects_wedge_weights() {
        // not a precise test of distribution -- just checks if in the ballpark
        let spinner = Spinner::new(vec![
            Wedge::new("Heads", 10, true),
            Wedge::new("Tails", 1, true),
        ]);
        let mut head_count = 0;
        let mut tail_count = 0;
        for _ in 1..1000 {
            match spinner.spin().unwrap() {
                "Heads" => head_count += 1,
                "Tails" => tail_count += 1,
                _ => panic!("unexpected value returned from spin()"),
            }
        }
        assert!(head_count > tail_count * 6);
    }

    #[test]
    fn spin_returns_none_if_selected_wedge_inactive() {
        let spinner = Spinner::new(vec![
            Wedge::new("Inactive", 1, false),
            Wedge::new("Also Inactive", 1, false),
        ]);
        for _ in 1..100 {
            assert!(spinner.spin().is_none());
        }
    }
}
