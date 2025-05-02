//! # Dice Module
//!
//! This module provides basic support for simulating rolls of standard polyhedral dice,
//! like d6, d10, d20, or any die with a custom number of sides (up to 255).
//!
//! The primary type is [`Die`], which lets you create and roll individual dice.
//! For rolling multiple dice at once and working with the results, see [`DicePool`].
//!
//! # Examples
//!
//! ## Roll a single die
//! ```
//! use gametools::Die;
//!
//! let d6 = Die::new(6);
//! let value = d6.roll();
//! assert!((1..=6).contains(&value));
//! ```
//!
//! ## Create and manipulate a dice pool
//! ```
//! use gametools::{Die, DicePool};
//!
//! let d10 = Die::new(10);
//!
//! // Roll 5d10 and create a DicePool
//! let pool = d10.roll_into_pool(5);
//!
//! // Most of the dicepool API is chainable...
//! let total = pool
//!     .reroll_if(&d10,|roll| roll == 1)   // re-rolls all 1's
//!     .buff(2)                            // increases all rolls by 2
//!     // .nerf(2)                         // (alternative: reduce all by 2)
//!     .take_highest(4)                    // take the highest 4, discarding 1
//!     .sum();                             // calculate the total
//!
//! assert!(total >= 12 && total <= 48); // 5(d10+2), drop lowest = 4(d10+2)
//! ```
//!
//! # Panics
//!
//! Creating a die with zero sides will panic:
//!
//! ```should_panic
//! use gametools::Die;
//! let invalid = Die::new(0);  // panics
//! ```
//!
//! Attempting to roll zero dice into a dicepool will also panic:
//!
//! ```should_panic
//! use gametools::Die;
//! let d6 = Die::new(5);
//! let pool = d6.roll_into_pool(0);    // panic!
//! ```

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A single die with a user-defined number of sides
pub struct Die {
    pub sides: u8,
}
impl Die {
    /// Creates a new die with the specified number of sides, up to 255 (u8).
    ///
    /// ## Panics
    /// - Panics if you try to create a Die with zero sides.
    ///
    /// ```should_panic
    /// use gametools::Die;
    /// let d6 = Die::new(6);
    /// let d20 = Die::new(20);
    ///
    /// let d0 = Die::new(0);  // panic!
    ///
    /// ```
    pub fn new(sides: u8) -> Die {
        assert!(sides > 0, "a Die with zero sides cannot be created");
        Die { sides }
    }

    /// Rolls the die and returns the face-up value.
    ///
    /// ```
    /// # use gametools::Die;
    /// let d10 = Die::new(10);
    /// let value = d10.roll();
    /// assert!((1..=10).contains(&value));
    /// ```
    pub fn roll(&self) -> u8 {
        rand::random_range(1..=self.sides)
    }

    /// Rolls the die multiple times and returns results as a DicePool.
    ///
    /// ## Panics
    /// - panics on attempt to roll zero dice to create a pool
    ///
    /// ```should_panic
    /// use gametools::{Die, DicePool};
    ///
    /// // create a pool of ten d10s
    /// let d10 = Die::new(10);
    /// let d10_pool = d10.roll_into_pool(10);
    /// assert_eq!(d10_pool.size(), 10);
    ///
    /// let no_dice = d10.roll_into_pool(0);    // this will panic!
    /// ```
    pub fn roll_into_pool(&self, times: usize) -> DicePool {
        assert!(
            times != 0,
            "cannot create a DicePool with zero dice (Die::roll_into_pool(0))"
        );
        DicePool {
            rolls: (0..times).map(|_| self.roll()).collect(),
        }
    }

    /// Rolls the die one and explodes (rolls again automatically and recurrently)
    /// if the specified trigger number is rolled.
    ///
    /// The value returned is maxed at 255 so that exploding dice results can still
    /// be used in a DicePool. Even with a d20, it would take rolling 13 consecutive 20s to hit the cap.
    ///
    /// ```
    /// use gametools::Die;
    ///
    /// let d6 = Die::new(6);
    ///
    /// // Note: an exploding die can never return the value it "explodes" on, because
    /// // it will always trigger another roll that will add at least 1 to it. Here we'll
    /// // explode the d6 on 5 -- in 1000 rolls it will never come up 5.
    /// for _ in 1..1000 {
    ///     let result = d6.roll_explode_on(5);
    ///     assert_ne!(result, 5)
    /// }
    /// ```
    pub fn roll_explode_on(&self, trigger: u8) -> u8 {
        let mut total = self.roll();
        if total == trigger {
            total = total.saturating_add(self.roll_explode_on(trigger));
        }
        total
    }

    /// Shortcut to the most common case: where a die "explodes" when the maximum is rolled.
    /// (6 on a d6, 20 on a d20, etc.)
    /// ```
    /// use gametools::Die;
    ///
    /// let d10 = Die::new(10);
    ///
    /// // A die that explodes on a maximum trigger value (e.g. 10 on a 10-sided die) can never
    /// // roll any multiple of that trigger value (e.g. 10, 20, 30, etc.)
    /// for _ in 1..10_000 {
    ///     let result = d10.roll_exploding();
    ///     assert!(result % 10 != 0);
    /// }
    /// ```
    pub fn roll_exploding(&self) -> u8 {
        self.roll_explode_on(self.sides)
    }
}

/// A pool of dice of a single die type (e.g. d6, d20).
///
/// This is considered as a group of n > 0 dice all simultaneously rolled. The collection containing
/// the individual die states (rolls) is not guaranteed to maintain any particular order. For
/// game logic where the order of results counts, it is generally better to get the rolls on demand
/// through roll() or roll_exploding().
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DicePool {
    rolls: Vec<u8>,
}

impl Default for DicePool {
    fn default() -> Self {
        Self::new()
    }
}
impl From<&[u8]> for DicePool {
    fn from(rolls: &[u8]) -> Self {
        Self {
            rolls: rolls.to_vec(),
        }
    }
}
impl From<Vec<u8>> for DicePool {
    fn from(rolls: Vec<u8>) -> Self {
        Self { rolls }
    }
}

impl DicePool {
    /// Creates a new, empty DicePool
    pub fn new() -> DicePool {
        DicePool {
            rolls: Vec::<u8>::new(),
        }
    }

    /// Returns a slice of all rolls in the pool.
    pub fn results(&self) -> &[u8] {
        &self.rolls
    }

    /// Adds a roll (u8) to the pool.
    pub fn add_roll(&mut self, roll: u8) {
        self.rolls.push(roll)
    }

    /// Returns sum of all die rolls in the pool.
    pub fn sum(&self) -> u64 {
        self.rolls.iter().map(|x| *x as u64).sum()
    }

    /// Returns number of die rolls in the pool.
    pub fn size(&self) -> usize {
        self.rolls.len()
    }

    /// Adds a buff / bonus to all rolls in the pool, with a maximum of 255 (u8_max).
    pub fn buff(&self, bonus: u8) -> Self {
        let buffed_rolls = self
            .rolls
            .iter()
            .map(|roll| roll.saturating_add(bonus))
            .collect::<Vec<u8>>();

        Self {
            rolls: buffed_rolls,
        }
    }

    /// Nerfs / reduces all rolls in the pool by the specified amount
    /// with a minimum of zero.
    pub fn nerf(&self, penalty: u8) -> Self {
        let nerfed_rolls = self
            .rolls
            .iter()
            .map(|roll| roll.saturating_sub(penalty))
            .collect::<Vec<u8>>();

        Self {
            rolls: nerfed_rolls,
        }
    }

    /// Returns an tuple in an Option::Some((min, max)) of the rolls in the pool, or None if the pool is empty
    /// or no minimum or maximum can be determined.
    pub fn range(&self) -> Option<(u8, u8)> {
        // ? operator on iter().max() takes care of the empty pool case for us
        let max = self.rolls.iter().max()?;
        let min = self.rolls.iter().min()?;
        Some((*min, *max))
    }

    /// Counts the number of times a particular value was rolled in the pool
    pub fn count_roll(&self, value: u8) -> usize {
        self.rolls.iter().filter(|&r| *r == value).count()
    }

    /// Returns a hashmap of "binned" values from the dicepool, where bins[value] = # of times that value was rolled.
    /// ```
    /// use gametools::DicePool;
    ///
    /// let some_rolls: &[u8] = &[2, 1, 1, 2];
    /// let pool: DicePool = some_rolls.into();
    /// let bins = pool.binned_rolls();
    ///
    /// assert_eq!(bins[&2], 2);
    /// assert_eq!(bins[&1], 2);
    /// assert!(bins.get(&5).is_none());
    /// ```
    pub fn binned_rolls(&self) -> HashMap<u8, usize> {
        let mut bins = HashMap::new();
        for roll in &self.rolls {
            _ = bins.insert(*roll, self.count_roll(*roll));
        }
        bins
    }

    /// Returns a new pool with only the highest-scoring 'n' rolls, discarding the rest.
    /// If n is zero, an empty pool is returned. If n is greater than the pool size, an
    /// unchanged (cloned) pool is returned.
    pub fn take_highest(&self, count: usize) -> Self {
        match count {
            0 => DicePool::new(),
            _ if count < self.size() => {
                let mut best_rolls = self.rolls.clone();
                best_rolls.sort_unstable_by(|a, b| b.cmp(a));
                best_rolls.truncate(count);
                best_rolls.into()
            }
            _ => self.clone(),
        }
    }

    /// Returns a new pool with only the lowest-scoring 'n' rolls, discarding the rest.
    /// If n is zero, an empty pool is returned. If n is greater than the pool size, an
    /// unchanged (cloned) pool is returned.
    pub fn take_lowest(&self, count: usize) -> Self {
        match count {
            0 => DicePool::new(),
            _ if count < self.size() => {
                let mut best_rolls = self.rolls.clone();
                best_rolls.sort_unstable();
                best_rolls.truncate(count);
                best_rolls.into()
            }
            _ => self.clone(),
        }
    }

    /// Rerolls any result that meets predicate criteria
    pub fn reroll_if<F>(&self, die: &Die, predicate: F) -> DicePool
    where
        F: Fn(u8) -> bool,
    {
        let rolls = &self.rolls;
        let rerolled: Vec<u8> = rolls
            .iter()
            .map(|&r| if predicate(r) { die.roll() } else { r })
            .collect();

        DicePool::from(rerolled)
    }

    /// Counts the number of rolls in the pool that meet the specified criteria.
    pub fn count_if<F>(&self, predicate: F) -> usize
    where
        F: Fn(u8) -> bool,
    {
        self.rolls.iter().filter(|r| predicate(**r)).count()
    }

    /// Counts the number of rolls in the pool over a specified threshold
    /// value.
    ///
    /// This is a convenience function that simply calls count_success_using with the
    /// appropriate closure.
    pub fn count_over(&self, threshold: u8) -> usize {
        self.count_if(|r| r > threshold)
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::*;

    #[test]
    fn create_die() {
        let d = Die::new(6);

        assert_eq!(d, Die { sides: 6 });
    }

    #[test]
    fn die_rolls_are_in_range() {
        let d4 = Die::new(4);
        for _ in 1..100 {
            let roll = d4.roll();
            assert!((1..=4).contains(&roll), "d4.roll() returned {}", roll);
        }

        let d20 = Die::new(20);
        for _ in 1..100 {
            let roll = d20.roll();
            assert!((1..=20).contains(&roll), "d20.roll() returned {}", roll);
        }
    }

    #[test]
    fn die_rolls_cover_all_sides() {
        let d20 = Die::new(20);
        let mut rolled = [false; 20];
        for _ in 0..=10_000 {
            rolled[(d20.roll() - 1) as usize] = true;
        }
        for (i, value_rolled) in rolled.iter().enumerate() {
            assert!(*value_rolled, "value {} was never rolled on a d20", i + 1);
        }
    }

    #[test]
    fn die_roll_into_pool_returns_correct_dicepool() {
        let d6 = Die::new(6);
        let d6_pool = d6.roll_into_pool(20);
        let rolls = d6_pool.results();
        // checks right number of rolls and that all are in expected range
        assert_eq!(rolls.len(), 20);
        for roll in rolls {
            assert!(
                (1..=6).contains(roll),
                "DicePool contained invalid d6 roll ({})",
                roll
            );
        }
    }

    #[test]
    #[should_panic]
    fn roll_zero_dice_into_pool_panics() {
        let d4 = Die::new(4);
        let _ = d4.roll_into_pool(0); // panic!
    }

    #[test]
    fn die_roll_explode_on_works() {
        // a d6 that explodes on 6 cannot roll a six... because that immediately
        // causes another roll that's at least 1, making an end result of 6 impossible.
        // so, for an exploding d6, all rolls should be in range 1..=5 or >= 7.
        let d6 = Die::new(6);
        let mut can_roll_over_die_max = false;

        // exploding roll enough times that we're sure we'll have at least one explode
        for _ in 1..=10000 {
            let roll = d6.roll_explode_on(6);
            assert!(
                roll != 6,
                "exploding d6 rolled a six -- should be impossible!"
            );
            if roll > 6 {
                can_roll_over_die_max = true;
            }
        }
        assert!(
            can_roll_over_die_max,
            "no values over max (# sides) returned from exploding roll"
        )
    }

    #[test]
    fn die_roll_exploding_works() {
        let sides = 12;
        let die = Die::new(sides);
        let mut can_roll_over_die_max = false;

        // exploding roll enough times that we're sure we'll have at least one explode
        for _ in 1..=10000 {
            // a die that explodes on n, where n in the max roll, can never roll any multiple of n
            let roll = die.roll_exploding();
            assert!(
                roll % die.sides != 0,
                "exploding d{} rolled a {} -- should be impossible!",
                die.sides,
                roll
            );

            // die should be able to roll higher than max sides when it explodes, too
            if roll > die.sides {
                can_roll_over_die_max = true;
            }
        }
        assert!(
            can_roll_over_die_max,
            "no values over non-exploding max (= # sides) ever returned from exploding roll"
        )
    }

    #[test]
    fn create_empty_dicepool() {
        let dp = DicePool::new();
        let rolls = dp.results();
        assert!(rolls.is_empty(), "new dicepool did not have empty results!");
    }

    #[test]
    fn create_empty_dicepool_via_default() {
        let pool = DicePool::default();
        assert_eq!(pool, DicePool::new());
    }

    #[test]
    fn create_dicepool_from_slice() {
        let some_rolls: &[u8] = &[21, 12];
        let pool_from_slice: DicePool = some_rolls.into();
        let mut test_pool = DicePool::new();
        test_pool.add_roll(21);
        test_pool.add_roll(12);
        assert_eq!(pool_from_slice, test_pool);
    }

    #[test]
    fn create_dicepool_from_vec_u8() {
        let some_rolls: Vec<u8> = vec![21u8, 12];
        let pool_from_vec: DicePool = some_rolls.into();
        let mut test_pool = DicePool::new();
        test_pool.add_roll(21);
        test_pool.add_roll(12);
        assert_eq!(pool_from_vec, test_pool);
    }

    #[test]
    fn binned_rolls_returns_correct_hashmap() {
        let some_rolls: &[u8] = &[2, 1, 1, 2, 9, 0, 1, 2, 5];
        let dp: DicePool = some_rolls.into();
        let bins = dp.binned_rolls();
        assert_eq!(bins[&2], 3);
        assert_eq!(bins[&1], 3);
        assert_eq!(bins[&9], 1);
        assert_eq!(bins[&0], 1);
        assert_eq!(bins[&5], 1);
        assert!(bins.get(&3).is_none());
    }

    #[test]
    fn add_roll_to_dicepool() {
        let mut dp = DicePool::new();
        dp.add_roll(1);
        assert_eq!(dp.results().len(), 1);
        assert_eq!(dp.results(), [1u8]);
    }

    #[test]
    fn dicepool_size_is_correct() {
        let mut dp = DicePool::new();
        assert_eq!(dp.size(), 0);
        dp.add_roll(21);
        dp.add_roll(12);
        assert_eq!(dp.size(), 2);
    }

    #[test]
    fn sum_rolls_in_dicepool() {
        let some_rolls = [1u8, 2, 3, 4];
        let mut dp = DicePool::new();
        for roll in some_rolls {
            dp.add_roll(roll);
        }
        assert_eq!(dp.sum(), 10);
    }

    #[test]
    fn dicepool_buff_works() {
        let some_rolls = [1u8, 2, 3, 255];
        let mut dp = DicePool::new();
        for roll in some_rolls {
            dp.add_roll(roll);
        }
        let buffed_dp = dp.buff(3);
        assert_eq!(buffed_dp.results(), [4u8, 5, 6, 255]);
    }

    #[test]
    fn dicepool_nerf_works() {
        let some_rolls = [1u8, 2, 3];
        let mut dp = DicePool::new();
        for roll in some_rolls {
            dp.add_roll(roll);
        }
        let nerfed_pool = dp.nerf(2);
        assert_eq!(nerfed_pool.results(), [0u8, 0, 1]);
    }

    #[test]
    fn dicepool_range_is_correct() {
        let mut dp = DicePool::new();
        assert_eq!(dp.range(), None);

        let some_rolls = [21, 12, 90, 125];
        for roll in some_rolls {
            dp.add_roll(roll);
        }
        assert_eq!(dp.range(), Some((12, 125)));
    }

    #[test]
    fn dicepool_range_returns_none_if_empty() {
        let pool = DicePool::new();
        assert!(pool.range().is_none());
    }

    #[test]
    fn dicepool_count_roll_works() {
        let some_rolls: &[u8] = &[2, 1, 1, 2, 1, 1, 2];
        let dp: DicePool = some_rolls.into();
        assert_eq!(dp.count_roll(2), 3);
        assert_eq!(dp.count_roll(1), 4);
        assert_eq!(dp.count_roll(6), 0);
    }

    #[test]
    fn dicepool_take_highest_works() {
        let some_rolls = vec![5, 3, 2, 4, 1u8];
        let dp: DicePool = some_rolls.into();

        let take_3 = dp.take_highest(3);
        assert_eq!(take_3.results(), [5, 4, 3]);

        let take_0 = dp.take_highest(0);
        assert_eq!(take_0.results(), []);

        let take_too_many = dp.take_highest(1_000_000);
        assert_eq!(take_too_many.results(), [5, 3, 2, 4, 1])
    }

    #[test]
    fn dicepool_take_lowest_works() {
        let some_rolls = vec![5, 2, 1, 3, 4u8];
        let dp: DicePool = some_rolls.into();

        let take_3 = dp.take_lowest(3);
        assert_eq!(take_3.results(), [1, 2, 3]);

        let take_0 = dp.take_lowest(0);
        assert_eq!(take_0.results(), []);

        let take_too_many = dp.take_lowest(1_000_000);
        assert_eq!(take_too_many.results(), [5, 2, 1, 3, 4])
    }

    #[test]
    fn dicepool_reroll_if_replaces_values_correctly() {
        let one_sided_die = Die::new(1); // always rolls a 1
        let some_rolls = vec![3, 2, 1, 1, 2u8];
        let dp = DicePool::from(some_rolls);

        let rerolled_twos = dp.reroll_if(&one_sided_die, |r| r == 2);
        assert_eq!(rerolled_twos.results(), &[3, 1, 1, 1, 1])
    }

    #[test]
    fn dicepool_count_success_using_works() {
        let some_rolls = vec![7, 7, 7, 8, 8, 8, 9, 9, 9];
        let pool = DicePool::from(some_rolls);

        let rolls_over_8 = pool.count_if(|r| r > 8);
        let even_rolls = pool.count_if(|r| r % 2 == 0);
        let rolled_7_or_9 = pool.count_if(|r| r == 7 || r == 9);

        assert_eq!(rolls_over_8, 3);
        assert_eq!(even_rolls, 3);
        assert_eq!(rolled_7_or_9, 6);
    }

    #[test]
    fn dicepool_count_success_over_is_correct() {
        let some_rolls = vec![7, 7, 7, 8, 8, 8, 9, 9, 9];
        let pool = DicePool::from(some_rolls);
        let success_threshold = 7;
        let successes = pool.count_over(success_threshold);
        assert_eq!(successes, 6);

        let success_threshold = 10;
        let successes = pool.count_over(success_threshold);
        assert_eq!(successes, 0);
    }
}
