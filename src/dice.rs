use rand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A single die with a user-defined number of sides
pub struct Die {
    pub sides: u8,
}
impl Die {
    /// Creates a new die with the specified number of sides, up to 256 (u8). 
    pub fn new(sides: u8) -> Die {
        Die { sides }
    }
    /// Rolls the die and returns the face-up value.
    pub fn roll(&self) -> u8 {
        rand::random_range(1..=self.sides)
    }
    /// Rolls the die multiple times and returns results as a DicePool.
    pub fn roll_many(&self, times: u64) -> DicePool {
        DicePool {
            rolls: (0..times).map(|_| self.roll()).collect()
        }
    }
}

/// A pool of multiple rolls of a single die type (e.g. d6, d20)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DicePool {
    rolls: Vec<u8>,
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
    /// Adds a roll to the pool.
    pub fn add_die(&mut self, roll: u8) {
        self.rolls.push(roll)
    }
    /// Returns sum of all die rolls in this pool.
    pub fn sum(&self) -> u64 {
        self.rolls.iter().map(|x| *x as u64).sum()
    }
}
