//! Spinners Module
//!
//! Implements a game Spinner, comprised of Wedges which can be uniform or of different
//! relative widths, and can be blocked / covered according to game conditions. Wedges
//! may contain numeric values, strings, enums, or other user-defined types (so long as
//! they implement Clone and PartialEq). Spinner methods largely return new Spinners
//! with the requested changes, rather than modifying the original spinner. This is
//! to allow for functional (chainable) programming patterns, and to avoid mutable state.
//!
//! ## Example
//! ```
//! # use gametools::spinners::{Wedge, Spinner, wedges_from_values};
//! let wedges = wedges_from_values(vec!["Rock", "Paper", "Scissors"]);
//! let rps = Spinner::new(wedges);
//! if let Some(spin) = rps.spin() {
//!     println!("You shoot: {spin}!");
//! }
//! // too easy, let's play like Sheldon Cooper instead!
//! let sheldonized_rps = rps
//!     .add_wedge(Wedge::new("Lizard"))
//!     .add_wedge(Wedge::new("Spock"));
//!
//! if let Some(spin) = sheldonized_rps.spin() {
//!     println!("You shoot: {spin}!");
//! }
//!
//! ```
//! ## Example
//! ```
//! use gametools::spinners::{Spinner, Wedge};
//! let spinner = Spinner::new(vec![
//!     Wedge::new_weighted("Heads", 75),
//!     Wedge::new_weighted("Tails", 25),
//! ]);
//! let toss = spinner.spin().unwrap();  // will be "Heads" 75% of the time
//! ```
//!
//! ## Example
//! ```
//! use gametools::spinners::{Spinner, Wedge};
//! let spinner = Spinner::new(vec![
//!     Wedge::new("Red"),
//!     Wedge::new("Blue"),
//!     Wedge::new("Green"),
//!     Wedge::new("Red"),
//! ]);
//!
//! // create a new spinner with "Red" wedges covered
//! // (blocks spinner from returning this value when landing on it, returns None instead)
//! let new_spinner = spinner.cover("Red");
//! for _ in 1..100 {
//!     if let Some(val) = new_spinner.spin() {
//!         assert_ne!(val, "Red");
//!         assert!(["Blue", "Green"].contains(&val));
//!     }
//! }
//! ```
//!
//! ## Example
//! ```
//! use gametools::spinners::{Spinner, Wedge};
//! let spinner = Spinner::new(vec![
//!    Wedge::new_weighted("Red", 2).cover(),   // start with all covered
//!    Wedge::new_weighted("Blue", 2).cover(),
//!    Wedge::new_weighted("Green", 2).cover(),
//! ]);
//! let new_spinner = spinner.uncover("Red");
//! // should now only be able to return Some("Red") or None
//! for _ in 1..100 {
//!     if let Some(val) = new_spinner.spin() {
//!         assert_eq!(val, "Red");
//!     }
//! }
//! ```
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

/// Creates a Vec of equally weighted (width = 1) Wedges from a Vec of values.
/// ```
/// use gametools::spinners::{wedges_from_values, Wedge};
/// let wedges = wedges_from_values(vec!["A", "B", "C"]);
/// assert_eq!(wedges.len(), 3);
/// assert_eq!(wedges[0], Wedge::new("A"));
/// ```
pub fn wedges_from_values<T: Clone>(values: Vec<T>) -> Vec<Wedge<T>> {
    values.into_iter().map(Wedge::new).collect()
}

/// Creates a Vec of weighted Wedges from (value, width) tuples.
/// ```
/// use gametools::spinners::{wedges_from_tuples, Wedge};
/// let wedges = wedges_from_tuples(vec![("Small",1), ("Medium",2), ("Large",3)]);
/// assert_eq!(wedges.len(), 3);
/// assert_eq!(wedges[0], Wedge::new_weighted("Small", 1));
/// ```
pub fn wedges_from_tuples<T: Clone>(tuples: Vec<(T, usize)>) -> Vec<Wedge<T>> {
    tuples
        .into_iter()
        .map(|(v, w)| Wedge::new_weighted(v, w))
        .collect()
}

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
    pub fn new_weighted(value: T, width: usize) -> Self {
        Self {
            value,
            width,
            active: true,
        }
    }

    /// Creates a new wedge with commonly used defaults (width = 1, active = true).
    pub fn new(value: T) -> Self {
        Self {
            value,
            width: 1,
            active: true,
        }
    }

    /// Cover this wedge (blocks spinner from returning this value when landing on it.)
    pub fn cover(&self) -> Self {
        Self {
            value: self.value.clone(),
            width: self.width,
            active: false,
        }
    }

    /// Uncover this wedge (removes any block so value is returned if spinner lands on it.)
    pub fn uncover(&self) -> Self {
        Self {
            value: self.value.clone(),
            width: self.width,
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

    /// Obtain a vector of the wedges currently on the spinner.
    pub fn wedges(&self) -> Vec<Wedge<T>> {
        self.wedges.clone()
    }

    /// Obtain an iterator over the wedges currently on the spinner.
    pub fn iter(&self) -> impl Iterator<Item = &Wedge<T>> {
        self.wedges.iter()
    }

    /// Spins the spinner, returning Some(value) of the wedge it lands on.
    /// Returns `None` if there are no wedges, or if the wedge selected is inactive / covered.
    /// The probability of landing on a particular wedge is determine by its width.
    ///
    /// ## Example
    /// ```
    /// use gametools::spinners::{Spinner, Wedge};
    /// let loaded_coin = Spinner::new(vec![
    ///     Wedge::new_weighted("Heads", 75),
    ///     Wedge::new_weighted("Tails", 25),
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
    ///     Wedge::new("Red"),
    ///     Wedge::new("Green"),
    ///     Wedge::new("Blue"),
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
            .map(|w| match w.value == target_val {
                true => w.cover(),
                false => w.clone(),
            })
            .collect();
        Spinner::new(covered)
    }

    /// Covers (inactivates) all wedges on the spinner.
    pub fn cover_all(&self) -> Spinner<T> {
        let all_covered = self.wedges.iter().map(|w| w.cover()).collect();
        Spinner::new(all_covered)
    }

    /// Returns a new spinner after uncovering any wedges that match a target value.
    pub fn uncover(&self, target_val: T) -> Spinner<T> {
        // create and return a new spinner with active = true on target wedges
        let uncovered = self
            .wedges
            .iter()
            .map(|w| match w.value == target_val {
                true => w.uncover(),
                false => w.clone(),
            })
            .collect();
        Spinner::new(uncovered)
    }

    /// Uncover / (re)activate all wedges on the spinner.
    pub fn uncover_all(&self) -> Spinner<T> {
        let uncovered = self.wedges.iter().map(|w| w.uncover()).collect();
        Spinner::new(uncovered)
    }

    /// Add a wedge to an existing spinner.
    ///
    /// ```
    /// # use gametools::spinners::{Wedge, Spinner};
    /// let rps = Spinner::new(vec![
    ///     Wedge::new("Rock"),
    ///     Wedge::new("Paper"),
    ///     Wedge::new("Scissors"),
    /// ]);
    /// let sheldonized_rps = rps
    ///     .add_wedge(Wedge::new("Lizard"))
    ///     .add_wedge(Wedge::new("Spock"));
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

    /// Remove any wedges matching a particular value from the spinner.
    ///
    /// ```
    /// # use gametools::spinners::{Wedge, Spinner};
    /// let spinner = Spinner::new(vec![
    ///     Wedge::new("Lose"),
    ///     Wedge::new("Win"),
    ///     Wedge::new("Lose"),
    /// ]);
    ///
    /// let never_lose_again = spinner.remove_wedges("Lose");
    ///
    /// if let Some(spin) = never_lose_again.spin() {
    ///     assert_ne!(spin, "Lose");
    /// }
    /// ```
    pub fn remove_wedges(&self, value: T) -> Spinner<T> {
        let shrunken = self
            .wedges
            .clone()
            .into_iter()
            .filter(|w| w.value != value)
            .collect();
        Spinner::new(shrunken)
    }

    /// Replaces a wedge value with another. Affects all wedges with that value.
    pub fn replace_value(&self, match_val: T, new_val: T) -> Spinner<T> {
        let updated = self
            .wedges
            .clone()
            .into_iter()
            .map(|w| match w.value == match_val {
                true => Wedge::new_weighted(new_val.clone(), w.width),
                false => w,
            })
            .collect();
        Spinner::new(updated)
    }
}

#[cfg(test)]
mod spinner_tests {
    use crate::spinners::*;

    #[test]
    fn wedges_from_values_creates_expected_wedges() {
        let wedges = wedges_from_values(vec!["A", "B", "C"]);
        assert_eq!(wedges.len(), 3);
        assert_eq!(wedges[0], Wedge::new("A"));
        assert_eq!(wedges[1], Wedge::new("B"));
        assert_eq!(wedges[2], Wedge::new("C"));
    }

    #[test]
    fn wedges_from_tuples_creates_expected_wedges() {
        let wedges = wedges_from_tuples(vec![("A", 1), ("B", 2), ("C", 3)]);
        assert_eq!(wedges.len(), 3);
        assert_eq!(wedges[0], Wedge::new_weighted("A", 1));
        assert_eq!(wedges[1], Wedge::new_weighted("B", 2));
        assert_eq!(wedges[2], Wedge::new_weighted("C", 3));
    }

    #[test]
    fn can_create_wedges_with_varied_value_types() {
        let text_wedge = Wedge::new_weighted("Winner".to_string(), 1);
        assert_eq!(text_wedge.value, "Winner");
        let numeric = Wedge::new_weighted(10, 1);
        assert_eq!(numeric.value, 10);
    }

    #[test]
    fn wedge_new_default_returns_expected_values() {
        let bad_one = Wedge::new("Bankrupt!");
        assert_eq!(bad_one.width, 1);
        assert_eq!(bad_one.active, true);
        assert_eq!(bad_one.value, "Bankrupt!");
    }

    #[test]
    fn can_create_spinners_with_varied_wedge_types() {
        let num_wedges = vec![
            Wedge::new_weighted(100, 1),
            Wedge::new_weighted(200, 1),
            Wedge::new_weighted(500, 1),
        ];
        let numeric_spinner = Spinner::new(num_wedges);
        assert_eq!(numeric_spinner.wedges.len(), 3);

        let text_wedges = vec![
            Wedge::new_weighted("Lose a Turn".to_string(), 2),
            Wedge::new_weighted("Ahead 4".to_string(), 4),
            Wedge::new_weighted("Back 2".to_string(), 4),
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
            Wedge::new_weighted("Heads", 1),
            Wedge::new_weighted("Tails", 1),
        ]);
        for _ in 1..100 {
            assert!(spinner.spin().is_some());
        }
    }

    #[test]
    fn spin_returns_only_expected_values() {
        let spinner = Spinner::new(vec![
            Wedge::new_weighted(1, 1),
            Wedge::new_weighted(2, 1),
            Wedge::new_weighted(3, 1),
        ]);
        for _ in 1..1000 {
            assert!((1..=3).contains(&spinner.spin().unwrap()));
        }
    }

    #[test]
    fn spin_respects_wedge_weights() {
        // not a precise test of distribution -- just checks if in the ballpark
        let spinner = Spinner::new(vec![
            Wedge::new_weighted("Heads", 10),
            Wedge::new_weighted("Tails", 1),
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
            Wedge::new("Inactive").cover(),
            Wedge::new("Also Inactive").cover(),
        ]);
        for _ in 1..100 {
            assert!(spinner.spin().is_none());
        }
    }

    #[test]
    fn spinner_cover_inactivates_only_the_right_wedges() {
        let spinner = Spinner::new(vec![
            Wedge::new_weighted("Red", 2),
            Wedge::new_weighted("Blue", 2),
            Wedge::new_weighted("Green", 2),
            Wedge::new_weighted("Red", 2),
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
            Wedge::new_weighted("Red", 2).cover(),
            Wedge::new_weighted("Blue", 2).cover(),
            Wedge::new_weighted("Green", 2).cover(),
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
            Wedge::new("Win"),
            Wedge::new("Lose"),
            Wedge::new("Draw"),
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
        let spinner = Spinner::new(vec![Wedge::new(1), Wedge::new(2)]);
        for _ in 1..100 {
            if let Some(spin) = spinner.spin() {
                assert!([1, 2].contains(&spin));
            }
        }
        let spinner = spinner.add_wedge(Wedge::new(3));
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

    #[test]
    fn can_remove_wedges_matching_value_from_spinner() {
        let spinner = Spinner::new(vec![Wedge::new(0), Wedge::new(1), Wedge::new(1)]);
        let one_removed = spinner.remove_wedges(1);
        for _ in 1..100 {
            match one_removed.spin() {
                Some(spin) => assert_eq!(spin, 0),
                None => panic!(
                    "spin should not return None if at least one active wedge is on the spinner"
                ),
            }
        }
    }

    #[test]
    fn can_obtain_copy_of_wedges_from_spinner() {
        let spinner = Spinner::new(vec![Wedge::new(1), Wedge::new(2)]);
        let wedges = spinner.wedges();
        let values: Vec<i32> = wedges.iter().map(|w| w.value).collect();
        assert_eq!(values, vec![1, 2]);
    }

    #[test]
    fn can_use_iterator_over_spinner_wedges() {
        let spinner = Spinner::new(vec![Wedge::new(1), Wedge::new(2)]);
        for wedge in spinner.iter() {
            assert!((1..=2).contains(&wedge.value));
        }
        assert_eq!(spinner.iter().count(), 2);
    }

    #[test]
    fn can_replace_values_on_spinner_wedges() {
        let rush_albums = Spinner::new(vec![
            Wedge::new("2112"),
            Wedge::new("Signals"),
            Wedge::new("Sheik Yerbouti"), // oops, that's Zappa
        ]);
        let rush_albums = rush_albums.replace_value("Sheik Yerbouti", "Power Windows");
        for _ in 1..100 {
            assert!(["2112", "Signals", "Power Windows"].contains(&rush_albums.spin().unwrap()))
        }
    }
}
