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
}
