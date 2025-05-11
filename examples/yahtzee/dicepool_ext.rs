use gametools::{DicePool, Die};

/// Extends gametools::DicePool to hash binned values, allow selective re-rolls and
/// conveniently build a DicePool from a slice.
pub trait DicePoolExt {
    fn reroll_by_idx(&self, indices: &[usize]) -> DicePool;
}

impl DicePoolExt for DicePool {
    /// Takes a list of indices for dice to re-roll and returns a new DicePool with
    /// new (but not necessarily different!) values for the indicated dice.
    fn reroll_by_idx(&self, indices: &[usize]) -> DicePool {
        let d6 = Die::new(6);
        let new_rolls: Vec<_> = self
            .results()
            .iter()
            .enumerate()
            .map(|(idx, orig_roll)| match indices.contains(&idx) {
                true => d6.roll(),
                false => *orig_roll,
            })
            .collect();

        new_rolls.into()
    }
}
