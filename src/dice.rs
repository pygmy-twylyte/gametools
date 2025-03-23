use rand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A single die with a user-defined number of sides
pub struct Die {
    pub sides: u8,
}
impl Die {
    pub fn new(sides: u8) -> Die {
        Die { sides }
    }
    pub fn roll(&self) -> u8 {
        rand::random_range(1..=self.sides)
    }
}

/// A pool of multiple rolls of a single die type (e.g. d6, d20)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DicePool {
    rolls: Vec<u8>,
}
impl DicePool {
    pub fn results(&self) -> &[u8] {
        &self.rolls
    }
}