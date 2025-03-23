pub mod dice;

#[cfg(test)]
mod tests {
    use super::dice::*;

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
    fn die_roll_many_returns_correct_dicepool() {
        let d6 = Die::new(6);
        let d6_pool = d6.roll_many(20);
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
    fn create_empty_dicepool() {
        let dp = DicePool::new();
        let rolls = dp.results();
        assert!(rolls.is_empty(), "new dicepool did not have empty results!")
    }
    #[test]
    fn add_roll_to_dicepool() {
        let mut dp = DicePool::new();
        dp.add_die(1);
        assert_eq!(dp.results().len(), 1);
        assert_eq!(dp.results(), [1u8]);
    }
    #[test]
    fn sum_rolls_in_dicepool() {
        let some_rolls = [1u8, 2, 3, 4];
        let mut dp = DicePool::new();
        for roll in some_rolls {
            dp.add_die(roll);
        }
        assert_eq!(dp.sum(), 10);
    }
}
