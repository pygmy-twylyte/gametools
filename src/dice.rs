//! # `Dice` - types and manipulations commonly needed for die-based games
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

    /// Create an exploding `Die` with spcified number of sides and trigger value.
    ///
    /// # Errors
    /// - Returns `DiceError::DieWithNoSides` if `sides` is zero.
    /// - Returns `DiceError::InvalidExplodeTrigger` if `explode_on` is not between 1 and `sides`.
    pub fn exploding(sides: u64, explode_on: u64) -> DieResult<Self> {
        if sides == 0 {
            return Err(DiceError::DieWithNoSides);
        }
        if explode_on > sides || explode_on == 0 {
            return Err(DiceError::InvalidExplodeTrigger { explode_on, sides });
        }
        Ok(Self {
            sides,
            explode_on: Some(explode_on),
        })
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
#[derive(Debug, Clone)]
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
    use super::{Die, DieResult, Rolls};

    #[test]
    fn can_create_regular_die() -> DieResult<()> {
        let d6 = Die::new(6)?;
        assert_eq!(d6.sides, 6);
        assert!(d6.explode_on.is_none());
        Ok(())
    }
}
