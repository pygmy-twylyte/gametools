//! # `Dice` - types and manipulations commonly needed for die-based games
//!
//! ## Types
//! - `Die` - a single numeric die with arbitrary number of sides and optional exploding behavior
//! - `Rolls` - an immutable pool of results from `Die` rolls
//! - `DieResult<T>` - alias for `Result<T, DiceError>`
//!
//! ## Example
//! ```
//! use gametools::Die;
//!
//! let rolls = Die::new(20)?.roll_n(3);
//! assert_eq!(rolls.len(), 3);
//! assert!(rolls.iter().all(|roll| (1..=20).contains(roll)));
//! # Ok::<(), gametools::DiceError>(())
//! ```
//!
pub type DieResult<T> = Result<T, DiceError>;

use std::collections::BTreeMap;

use rand::random_range;

use crate::gameerror::DiceError;

/// A single `Die`.
///
/// All dice must have at least one side. If an `explode_on` trigger value
/// is specified and that value is rolled, the `Die` is rolled again and that result
/// is added to the final returned value. The additional rolls repeat and are summed
/// indefinitely as long as the trigger value keeps coming up (potentially "exploding"
/// if you're on a really hot streak.)
///

#[derive(Debug, Clone, Copy)]
pub struct Die {
    sides: u64,
    explode_on: Option<u64>,
}

impl Die {
    /// Create a non-exploding `Die` with specified number of sides.
    ///
    /// # Errors
    /// - Returns `DiceError::DieWithNoSides` if `sides` is zero.
    pub fn new(sides: u64) -> DieResult<Self> {
        if sides == 0 {
            return Err(DiceError::DieWithNoSides);
        }
        Ok(Self {
            sides,
            explode_on: None,
        })
    }

    /// Create a non-exploding `Die` with specified number of sides, unchecked for
    /// validity. Can be used in `const` contexts.
    pub const fn new_unchecked(sides: u64) -> Self {
        Self {
            sides,
            explode_on: None,
        }
    }

    /// Create an exploding `Die` with spcified number of sides and trigger value.
    ///
    /// # Errors
    /// - Returns `DiceError::DieWithNoSides` if `sides` is zero.
    /// - Returns `DiceError::InvalidExplodeTrigger` if `explode_on` is not between 1 and `sides`.
    pub fn exploding(sides: u64, explode_on: u64) -> DieResult<Self> {
        if sides == 0 {
            return Err(DiceError::DieWithNoSides);
        }
        if sides == 1 {
            return Err(DiceError::InfiniteExplosion);
        }
        if explode_on > sides || explode_on == 0 {
            return Err(DiceError::InvalidExplodeTrigger { explode_on, sides });
        }
        Ok(Self {
            sides,
            explode_on: Some(explode_on),
        })
    }

    /// Create a exploding `Die` with specified number of sides, unchecked for
    /// validity. Can be used in `const` contexts.
    pub const fn exploding_unchecked(sides: u64, explode_on: u64) -> Self {
        Self {
            sides,
            explode_on: Some(explode_on),
        }
    }

    /// Get the number of sides on this `Die`.
    #[must_use]
    pub fn sides(&self) -> u64 {
        self.sides
    }

    /// Get the explode trigger for this `Die`, if it has one.
    #[must_use]
    pub fn explode_on(&self) -> Option<u64> {
        self.explode_on
    }

    /// Roll this `Die` and return the result.
    #[must_use]
    pub fn roll(&self) -> u64 {
        let mut result = random_range(1..=self.sides);
        if let Some(trigger) = self.explode_on
            && trigger == result
        {
            let mut explode = true;
            while explode {
                let bonus_roll = random_range(1..=self.sides);
                result += bonus_roll;
                explode = bonus_roll == trigger;
            }
        }
        result
    }

    /// Roll this `Die` n times and return the collected results as `Rolls`.
    #[must_use]
    pub fn roll_n(&self, n: usize) -> Rolls {
        Rolls((0..n).map(|_| self.roll()).collect())
    }
}

/// A set of results from rolling dice.
#[derive(Debug, Clone, PartialEq)]
pub struct Rolls(Vec<u64>);
impl Rolls {
    /// Add all of the individual rolls together and return the result.
    #[must_use]
    pub fn sum(&self) -> u64 {
        self.0.iter().sum()
    }
    /// Return the number of rolls in this set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Return `true` if this set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Return an iterator over the rolls in this set.
    pub fn iter(&self) -> impl Iterator<Item = &u64> {
        self.0.iter()
    }
    /// Return a slice of the rolls in this set.
    #[must_use]
    pub fn as_slice(&self) -> &[u64] {
        &self.0
    }
    /// Return the maximum value rolled, or `None` if the set is empty.
    #[must_use]
    pub fn max(&self) -> Option<u64> {
        self.0.iter().max().copied()
    }
    #[must_use]
    /// Return the minimum value rolled, or `None` if the set is empty.
    pub fn min(&self) -> Option<u64> {
        self.0.iter().min().copied()
    }
    /// Return a histogram of the rolls in this set, mapping each value to the number of times it was rolled.
    #[must_use]
    pub fn histogram(&self) -> BTreeMap<u64, usize> {
        self.0.iter().fold(BTreeMap::new(), |mut histogram, roll| {
            *histogram.entry(*roll).or_insert(0) += 1;
            histogram
        })
    }
    /// Return a new set with the `count` highest rolls retained, sorted in descending order.
    #[must_use]
    pub fn highest(&self, count: usize) -> Rolls {
        let mut sorted = self.0.clone();
        sorted.sort_unstable();
        Rolls(sorted.into_iter().rev().take(count).collect())
    }
    /// Return a new set with the `count` lowest rolls retained, sorted in ascending order.
    #[must_use]
    pub fn lowest(&self, count: usize) -> Rolls {
        let mut sorted = self.0.clone();
        sorted.sort_unstable();
        Rolls(sorted.into_iter().take(count).collect())
    }
    /// Return a new set with the `count` highest rolls dropped, sorted in descending order.
    #[must_use]
    pub fn drop_highest(&self, count: usize) -> Rolls {
        let mut sorted = self.0.clone();
        sorted.sort_unstable();
        Rolls(sorted.into_iter().rev().skip(count).collect())
    }
    /// Return a new set with the `count` lowest rolls dropped, sorted in ascending order.
    #[must_use]
    pub fn drop_lowest(&self, count: usize) -> Rolls {
        let mut sorted = self.0.clone();
        sorted.sort_unstable();
        Rolls(sorted.into_iter().skip(count).collect())
    }
    /// Return the number of rolls in this set that satisfy the given `chooser` predicate.
    #[must_use]
    pub fn count_where<P>(&self, mut chooser: P) -> usize
    where
        P: FnMut(u64) -> bool,
    {
        self.0.iter().filter(|&i| chooser(*i)).count()
    }
}

impl From<&[u64]> for Rolls {
    fn from(value: &[u64]) -> Self {
        Self(value.to_vec())
    }
}
impl From<Vec<u64>> for Rolls {
    fn from(value: Vec<u64>) -> Self {
        Self(value)
    }
}

impl AsRef<[u64]> for Rolls {
    fn as_ref(&self) -> &[u64] {
        &self.0
    }
}

impl IntoIterator for Rolls {
    type Item = u64;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use crate::Rolls;

    use super::{DiceError, Die, DieResult};

    const N_TEST_ROLLS: usize = 100;

    #[test]
    fn can_create_regular_die() -> DieResult<()> {
        let d6 = Die::new(6)?;
        assert_eq!(d6.sides, 6);
        assert!(d6.explode_on.is_none());
        Ok(())
    }

    #[test]
    fn can_create_exploding_die() -> DieResult<()> {
        let die = Die::exploding(4, 4)?;
        assert_eq!(die.sides, 4);
        assert!(matches!(die.explode_on, Some(4)));
        Ok(())
    }

    #[test]
    fn creating_die_with_no_sides_yields_correct_error() {
        let die = Die::new(0);
        assert!(die.is_err_and(|e| matches!(e, DiceError::DieWithNoSides)));
        let exploder = Die::exploding(0, 0);
        assert!(exploder.is_err_and(|e| matches!(e, DiceError::DieWithNoSides)));
    }

    #[test]
    fn creating_exploding_die_with_one_side_yields_correct_error() {
        let exploder = Die::exploding(1, 1);
        assert!(exploder.is_err_and(|e| matches!(e, DiceError::InfiniteExplosion)));
    }

    #[test]
    fn creating_exploding_die_with_invalid_trigger_yields_correct_error() {
        let high_trigger = Die::exploding(4, 5);
        assert!(high_trigger.is_err_and(|e| matches!(
            e,
            DiceError::InvalidExplodeTrigger {
                explode_on: 5,
                sides: 4
            }
        )));
        let zero_trigger = Die::exploding(4, 0);
        assert!(zero_trigger.is_err_and(|e| matches!(
            e,
            DiceError::InvalidExplodeTrigger {
                explode_on: 0,
                sides: 4
            }
        )));
    }

    #[test]
    fn regular_dice_yield_correct_range() {
        let d4 = Die::new(4).unwrap();
        assert_eq!(d4.sides, 4);
        assert!(
            (0..N_TEST_ROLLS)
                .map(|_| d4.roll())
                .all(|r| r > 0 && r <= d4.sides)
        );
    }

    #[test]
    fn regular_dice_yield_every_expected_value() {
        let expected_vals = HashSet::from([1u64, 2, 3, 4, 5, 6]);
        let d6 = Die::new(6).unwrap();
        let actual_vals: HashSet<u64> = (0..N_TEST_ROLLS).map(|_| d6.roll()).collect();
        assert_eq!(expected_vals, actual_vals);
    }

    #[test]
    fn exploding_dice_sometimes_yield_higher_than_number_of_sides() {
        let d4_x4 = Die::exploding(4, 4).unwrap();
        assert!((0..N_TEST_ROLLS).map(|_| d4_x4.roll()).any(|roll| roll > 4));
    }

    #[test]
    fn exploding_die_cannot_yield_its_trigger_value() {
        let d4_x4 = Die::exploding(4, 4).unwrap();
        assert!(
            !(0..N_TEST_ROLLS)
                .map(|_| d4_x4.roll())
                .any(|roll| roll == 4)
        );
    }

    #[test]
    fn die_roll_n_returns_correct_rolls() -> DieResult<()> {
        let d4_rolls = Die::new(4)?.roll_n(N_TEST_ROLLS);
        assert_eq!(d4_rolls.len(), N_TEST_ROLLS);
        assert_eq!(d4_rolls.min().unwrap(), 1);
        assert_eq!(d4_rolls.max().unwrap(), 4);
        Ok(())
    }

    #[test]
    fn rolls_sum_is_correct() {
        let empty_rolls = Rolls::from(vec![]);
        assert_eq!(empty_rolls.sum(), 0);
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.sum(), 11);
    }

    #[test]
    fn rolls_len_is_correct() {
        let rolls = Rolls::from([1, 1, 1].as_ref());
        assert_eq!(rolls.len(), 3);
    }

    #[test]
    fn rolls_is_empty_is_correct() {
        let empty_rolls = Rolls::from(vec![]);
        assert!(empty_rolls.is_empty());
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert!(!d4_rolls.is_empty());
    }

    #[test]
    fn rolls_max_is_correct() {
        let empty_rolls = Rolls::from(vec![]);
        assert_eq!(empty_rolls.max(), None);
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.max(), Some(4));
    }

    #[test]
    fn rolls_min_is_correct() {
        let empty_rolls = Rolls::from(vec![]);
        assert_eq!(empty_rolls.min(), None);
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.min(), Some(1));
    }

    #[test]
    fn rolls_histogram_is_correct() {
        let expected: BTreeMap<u64, usize> = BTreeMap::from([(1, 2), (2, 1), (3, 1), (4, 1)]);
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.histogram(), expected);
    }

    #[test]
    fn rolls_highest_returns_right_rolls_in_descending_order() {
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.highest(2), Rolls::from(vec![4, 3]));
    }

    #[test]
    fn rolls_lowest_returns_right_rolls_in_ascending_order() {
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.lowest(3), Rolls::from(vec![1, 1, 2]));
    }

    #[test]
    fn rolls_drop_highest_returns_right_rolls_in_descending_order() {
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.drop_highest(2), Rolls::from(vec![2, 1, 1]));
    }

    #[test]
    fn rolls_drop_lowest_returns_right_rolls_in_ascending_order() {
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.drop_lowest(2), Rolls::from(vec![2, 3, 4]));
    }

    #[test]
    fn rolls_count_where_is_correct() {
        let d4_rolls = Rolls::from(vec![1, 3, 2, 4, 1]);
        assert_eq!(d4_rolls.count_where(|r| r == 1), 2);
        assert_eq!(d4_rolls.count_where(|r| r > 2), 2);
        assert_eq!(d4_rolls.count_where(|r| !r.is_multiple_of(2)), 3);
        assert_eq!(d4_rolls.count_where(u64::is_power_of_two), 4);
    }
}
