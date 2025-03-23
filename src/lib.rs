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
    fn create_dicepool_from_slice() {
        let some_rolls: &[u8] = &[21,12];
        let pool_from_slice: DicePool = some_rolls.into();
        let mut test_pool = DicePool::new();
        test_pool.add_roll(21);
        test_pool.add_roll(12);
        assert_eq!(pool_from_slice, test_pool);
    }
    #[test]
    fn create_dicepool_from_vec_u8() {
        let some_rolls: Vec<u8> = vec![21u8,12];
        let pool_from_vec: DicePool = some_rolls.into();
        let mut test_pool = DicePool::new();
        test_pool.add_roll(21);
        test_pool.add_roll(12);
        assert_eq!(pool_from_vec, test_pool);
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
        dp.buff(3);
        assert_eq!(dp.results(), [4u8, 5, 6, 255]);
    }
    #[test]
    fn dicepool_nerf_works() {
        let some_rolls = [1u8, 2, 3];
        let mut dp = DicePool::new();
        for roll in some_rolls {
            dp.add_roll(roll);
        }
        dp.nerf(2);
        assert_eq!(dp.results(), [0u8, 0, 1]);
    }
    #[test]
    fn dicepool_range_works() {
        let mut dp = DicePool::new();
        assert_eq!(dp.range(), None);

        let some_rolls = [21, 12, 90, 125];
        for roll in some_rolls {
            dp.add_roll(roll);
        }
        assert_eq!(dp.range(), Some((12,125)));
    }
}
