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

    /// Creates a new wedge with commonly used defaults (width = 1, active = true).
    pub fn new_default(value: T) -> Self {
        Self {
            value,
            width: 1,
            active: true,
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

impl<T: Clone + PartialEq + std::fmt::Debug> Spinner<T> {
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
        Some(chosen_wedge.value)
    }

    /// Returns a new spinner with a target value covered (blocked).
    /// Returns a clone of the original spinner if there is no wedge matching the target value.
    ///
    /// ## Example
    /// ```
    /// use gametools::spinners::{Wedge, Spinner};
    /// let original = Spinner::new(vec![
    ///     Wedge::new("Red", 1, true),
    ///     Wedge::new("Green", 1, true),
    ///     Wedge::new("Blue", 1, true),
    /// ]);
    /// let red_blocked = original.cover("Red");
    /// // red_blocked.spin() now returns None if the spinner lands on Red
    /// if let Some(color) = red_blocked.spin() {
    ///     assert!((color == "Green") | (color == "Blue"));
    /// }
    /// ```
    pub fn cover(&self, target_val: T) -> Spinner<T> {
        // create and return a new spinner with active = false on target wedges
        let covered = self
            .wedges
            .iter()
            .map(|w| {
                let mut new_wedge = w.clone();
                if w.value == target_val {
                    new_wedge.active = false;
                }
                new_wedge
            })
            .collect();
        Spinner::new(covered)
    }

    /// Covers (inactivates) all wedges on the spinner.
    pub fn cover_all(&self) -> Spinner<T> {
        let all_covered = self
            .wedges
            .iter()
            .map(|w| {
                let mut new_wedge = w.clone();
                new_wedge.active = false;
                new_wedge
            })
            .collect();
        Spinner::new(all_covered)
    }

    /// Returns a new spinner after uncovering any wedges that match a target value.
    pub fn uncover(&self, target_val: T) -> Spinner<T> {
        // create and return a new spinner with active = true on target wedges
        let uncovered = self
            .wedges
            .iter()
            .map(|w| {
                let mut new_wedge = w.clone();
                if w.value == target_val {
                    new_wedge.active = true;
                }
                new_wedge
            })
            .collect();
        Spinner::new(uncovered)
    }

    /// Uncover / (re)activate all wedges on the spinner.
    pub fn uncover_all(&self) -> Spinner<T> {
        let uncovered = self
            .wedges
            .iter()
            .map(|w| {
                let mut new_wedge = w.clone();
                new_wedge.active = true;
                new_wedge
            })
            .collect();
        Spinner::new(uncovered)
    }

    /// Add a wedge to an existing spinner.
    ///
    /// ```
    /// # use gametools::spinners::{Wedge, Spinner};
    /// let rps = Spinner::new(vec![
    ///     Wedge::new_default("Rock"),
    ///     Wedge::new_default("Paper"),
    ///     Wedge::new_default("Scissors"),
    /// ]);
    /// let sheldonized_rps = rps
    ///     .add_wedge(Wedge::new_default("Lizard"))
    ///     .add_wedge(Wedge::new_default("Spock"));
    ///
    /// if let Some(spin) = sheldonized_rps.spin() {
    ///     println!("You shoot: {spin}!");
    /// }
    ///
    /// ```
    pub fn add_wedge(&self, new_wedge: Wedge<T>) -> Spinner<T> {
        let mut added = self.wedges.clone();
        added.push(new_wedge);
        Spinner::new(added)
    }
}

#[cfg(test)]
mod spinner_tests {
    use crate::spinners::*;
    #[test]
    fn can_create_wedges_with_varied_value_types() {
        let text_wedge = Wedge::new("Winner".to_string(), 1, true);
        assert_eq!(text_wedge.value, "Winner");
        let numeric = Wedge::new(10, 1, true);
        assert_eq!(numeric.value, 10);
    }

    #[test]
    fn wedge_new_default_returns_expected_values() {
        let bad_one = Wedge::new_default("Bankrupt!");
        assert_eq!(bad_one.width, 1);
        assert_eq!(bad_one.active, true);
        assert_eq!(bad_one.value, "Bankrupt!");
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

    #[test]
    fn spinner_cover_inactivates_only_the_right_wedges() {
        let spinner = Spinner::new(vec![
            Wedge::new("Red", 2, true),
            Wedge::new("Blue", 2, true),
            Wedge::new("Green", 2, true),
            Wedge::new("Red", 2, true),
        ]);
        let new_spinner = spinner.cover("Red");
        for _ in 1..100 {
            if let Some(val) = new_spinner.spin() {
                assert_ne!(val, "Red");
                assert!(["Blue", "Green"].contains(&val));
            }
        }
    }

    #[test]
    fn spinner_uncover_activates_only_the_right_wedges() {
        // start with all covered
        let spinner = Spinner::new(vec![
            Wedge::new("Red", 2, false),
            Wedge::new("Blue", 2, false),
            Wedge::new("Green", 2, false),
        ]);
        let new_spinner = spinner.uncover("Red");
        // should now only be able to return Some("Red") or None
        for _ in 1..100 {
            if let Some(val) = new_spinner.spin() {
                assert_eq!(val, "Red");
            }
        }
    }

    #[test]
    fn uncover_all_and_cover_all_work_correctly() {
        let spinner = Spinner::new(vec![
            Wedge::new_default("Win"),
            Wedge::new_default("Lose"),
            Wedge::new_default("Draw"),
        ]);

        let all_covered = spinner.cover_all();
        for _ in 1..100 {
            assert!(all_covered.spin().is_none());
        }

        let all_uncovered = all_covered.uncover_all();
        for _ in 1..100 {
            assert!(all_uncovered.spin().is_some());
        }
    }

    #[test]
    fn can_add_wedge_to_existing_spinner() {
        let spinner = Spinner::new(vec![Wedge::new_default(1), Wedge::new_default(2)]);
        for _ in 1..100 {
            if let Some(spin) = spinner.spin() {
                assert!([1, 2].contains(&spin));
            }
        }
        let spinner = spinner.add_wedge(Wedge::new_default(3));
        let mut spun_a_3 = false;
        for _ in 1..1000 {
            if let Some(3) = spinner.spin() {
                spun_a_3 = true;
            }
        }
        assert!(
            spun_a_3,
            "new value not returned from spinner in 1000 spins"
        )
    }
}
