use rand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A single die with a user-defined number of sides
pub struct Die {
    pub sides: u8,
}
impl Die {
    /// Creates a new die with the specified number of sides, up to 255 (u8).
    /// Panics if you try to create a Die with no sides.
    pub fn new(sides: u8) -> Die {
        assert!(sides > 0, "a Die with zero sides cannot be created");
        Die { sides }
    }

    /// Rolls the die and returns the face-up value.
    pub fn roll(&self) -> u8 {
        rand::random_range(1..=self.sides)
    }

    /// Rolls the die multiple times and returns results as a DicePool.
    pub fn roll_into_pool(&self, times: usize) -> DicePool {
        DicePool {
            rolls: (0..times).map(|_| self.roll()).collect(),
        }
    }

    /// Rolls the die one and explodes (rolls again automatically and recurrently)
    /// if the specified trigger number is rolled.
    ///
    /// The value returned is maxed at 255 so that exploding dice results can still
    /// be used in a DicePool. Even with a d20, it would take rolling 13 consecutive 20s to hit the cap.
    pub fn roll_explode_on(&self, trigger: u8) -> u8 {
        let mut total = self.roll();
        if total == trigger {
            total = total.saturating_add(self.roll_explode_on(trigger));
        }
        total
    }

    /// Shortcut to the common case where a die "explodes" when the maximum is rolled (6 on a d6, 20 on a d20, etc.)
    pub fn roll_exploding(&self) -> u8 {
        self.roll_explode_on(self.sides)
    }
}

/// A pool of multiple rolls of a single die type (e.g. d6, d20)
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
        if self.rolls.is_empty() {
            return None;
        }
        let max = match self.rolls.iter().max() {
            Some(roll) => roll,
            None => {
                return None;
            }
        };
        let min = match self.rolls.iter().min() {
            Some(roll) => roll,
            None => {
                return None;
            }
        };

        Some((*min, *max))
    }
    /// Counts the number of times a particular value was rolled in the pool
    pub fn count_roll(&self, value: u8) -> usize {
        self.rolls.iter().filter(|&r| *r == value).count()
    }

    /// Returns a new pool with only the highest-scoring 'n' rolls, discarding the rest.
    /// If n is zero, an empty pool is returned. If n is greater than the pool size, an
    /// unchanged pool is returned.
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
    /// unchanged pool is returned.
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
        let rerolled: Vec<u8> = self
            .rolls
            .iter()
            .map(|&r| if predicate(r) { die.roll() } else { r })
            .collect();

        DicePool::from(rerolled)
    }

    /// Counts the number of rolls in the pool that meet a certain "success" criteria.
    pub fn count_success_using<F>(&self, predicate: F) -> usize 
    where
        F: Fn(u8) -> bool,
    {
        self.rolls
            .iter()
            .filter(|r| predicate(**r))
            .count()
    }

    /// Counts the number of rolls in the pool over a specified threshold 
    /// "success" value.
    /// 
    /// This is a convenience function that simply calls count_success_using with the
    /// appropriate closure.
    pub fn count_success_over(&self, threshold: u8) -> usize {
        self.count_success_using(|r| r > threshold)
    } 
}

