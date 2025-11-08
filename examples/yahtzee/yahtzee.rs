//! ** Yahtzee module **
//! Uses gametools::DicePool to manage dice, implement the scoring functions and rules,
//! and manage state for a game of Yahtzee.
use gametools::{DicePool, Die};
use std::collections::HashMap;

/// Holds and manages the state of the current Yahtzee game.
#[derive(Debug, Clone)]
pub struct GameState {
    pub turns_left: u8,
    pub rolls_left: u8,
    pub dice: DicePool,
    pub scorecard: YahtzeeScorecard,
}
impl GameState {
    /// Create a new standard game of Yahtzee with a set of five d6 and
    /// a blank scorecard.
    pub fn new_game() -> Self {
        let d6 = Die::new(6);
        Self {
            turns_left: 13,
            rolls_left: 3,
            dice: d6.roll_into_pool(5),
            scorecard: YahtzeeScorecard::new(),
        }
    }

    /// Obtain a list of open (no score yet) categories.
    /// Note that Yahtzee! is always possible and is always on the list.
    pub fn open_categories(&self) -> Vec<Categories> {
        let mut open_cats: Vec<Categories> = self
            .scorecard
            .scores
            .iter()
            .filter(|(_cat, score)| score.is_none())
            .map(|(cat, _score)| *cat)
            .collect();
        // Yahtzee is always scorable even if we already have one
        if !open_cats.contains(&Categories::Yahtzee) {
            open_cats.push(Categories::Yahtzee);
        }
        open_cats
    }

    /// Starts a turn by rolling all 5 dice.
    pub fn start_turn(&mut self) -> Result<(), &'static str> {
        // make sure there are turns left and one isn't already ongoing
        if self.turns_left == 0 {
            return Err("start_turn called when no turns remain");
        }
        if self.rolls_left < 3 {
            return Err("start_turn called while a turn was already in progress");
        }

        // roll all 5 dice
        let d6 = Die::new(6);
        self.dice = d6.roll_into_pool(5);

        // update game state to reflect turn started and first roll used
        self.rolls_left -= 1;
        self.turns_left -= 1;
        Ok(())
    }

    /// Takes a slice of indices into the DicePool, and re-rolls the dice corresponding to them.
    pub fn reroll_dice(&mut self, indices: &[usize]) -> Result<(), &'static str> {
        if self.rolls_left == 0 {
            return Err("no re-rolls remain this turn");
        }
        let d6 = Die::new(6);
        self.dice = self.dice.reroll_by_idx(&d6, indices);
        self.rolls_left -= 1;
        Ok(())
    }

    /// Returns the score the dice would get if played in a given category according to official
    /// Yahtzee game rules.
    ///
    /// If an empty slice is passed for rolls, the dicepool from the current "real" gamestate is used.
    /// If not empty, the passed "theoretical" rolls are used instead.
    /// Returns zero where the category doesn't apply.
    pub fn score_roll_as(&self, rolls: &[u8], category: Categories) -> usize {
        // use real game dice if 'rolls' is empty, or rolls parameter if not
        let dicepool = match rolls.is_empty() {
            true => self.dice.clone(),
            false => rolls.into(),
        };

        let binned_rolls = dicepool.binned_rolls();
        let mut sorted_rolls = dicepool.results().to_vec();
        sorted_rolls.sort();

        match category {
            Categories::Aces => dicepool.count_roll(1),
            Categories::Twos => dicepool.count_roll(2) * 2,
            Categories::Threes => dicepool.count_roll(3) * 3,
            Categories::Fours => dicepool.count_roll(4) * 4,
            Categories::Fives => dicepool.count_roll(5) * 5,
            Categories::Sixes => dicepool.count_roll(6) * 6,
            Categories::ThreeOfAKind => {
                if binned_rolls.values().any(|&count| count > 2) {
                    dicepool.sum() as usize
                } else {
                    0
                }
            }
            Categories::FourOfAKind => {
                if binned_rolls.values().any(|&count| count > 3) {
                    dicepool.sum() as usize
                } else {
                    0
                }
            }
            Categories::FullHouse => {
                let mut has_pair = false;
                let mut has_trio = false;
                for roll in 1..=6 {
                    match binned_rolls.get(&roll).unwrap_or(&0) {
                        3 => has_trio = true,
                        2 => has_pair = true,
                        _ => (),
                    }
                }
                if has_trio && has_pair { 25 } else { 0 }
            }
            Categories::SmallStraight => {
                sorted_rolls.dedup();
                let mut windows = sorted_rolls.windows(4);
                if windows.any(|window| {
                    window == [1, 2, 3, 4] || window == [2, 3, 4, 5] || window == [3, 4, 5, 6]
                }) {
                    30
                } else {
                    0
                }
            }
            Categories::LargeStraight => {
                if sorted_rolls == [1, 2, 3, 4, 5] || sorted_rolls == [2, 3, 4, 5, 6] {
                    40
                } else {
                    0
                }
            }
            Categories::Yahtzee => {
                if binned_rolls.values().any(|&count| count == 5) {
                    50
                } else {
                    0
                }
            }
            Categories::Chance => dicepool.sum() as usize,
        }
    }
    /// Displays the final scorecard with all scores and bonuses tallied
    pub fn show_final_score(&mut self) {
        // calculate sum of upper section
        self.scorecard.upper_subtotal = [
            Categories::Aces,
            Categories::Twos,
            Categories::Threes,
            Categories::Fours,
            Categories::Fives,
            Categories::Sixes,
        ]
        .iter()
        .map(|&cat| self.scorecard.scores[&cat].unwrap_or(0) as usize)
        .collect::<Vec<usize>>()
        .iter()
        .sum();

        // award upper section bonus if applicable
        if self.scorecard.upper_subtotal > 62 {
            self.scorecard.upper_bonus = 35;
            self.scorecard.upper_total = self.scorecard.upper_subtotal + 35;
        } else {
            self.scorecard.upper_bonus = 0;
            self.scorecard.upper_total = self.scorecard.upper_subtotal;
        }

        // calculate sum of lower section
        self.scorecard.lower_total = [
            Categories::ThreeOfAKind,
            Categories::FourOfAKind,
            Categories::FullHouse,
            Categories::SmallStraight,
            Categories::LargeStraight,
            Categories::Yahtzee,
            Categories::Chance,
        ]
        .iter()
        .map(|cat| self.scorecard.scores[cat].unwrap_or(0) as usize)
        .collect::<Vec<usize>>()
        .iter()
        .sum();

        // add any bonus for extra Yahtzees
        self.scorecard.lower_total += 100 * self.scorecard.extra_yahtzees;

        // calculate grand total
        self.scorecard.final_score = self.scorecard.upper_total + self.scorecard.lower_total;

        // print the completed scorecard
        println!("{}", self.scorecard);
    }
}

/// Each of the various scoring categories in Yahtzee.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Categories {
    Aces,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    ThreeOfAKind,
    FourOfAKind,
    FullHouse,
    SmallStraight,
    LargeStraight,
    Yahtzee,
    Chance,
}
impl std::fmt::Display for Categories {
    /// Sets text representation for each of the categories.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Categories::Aces => write!(f, "Aces"),
            Categories::Twos => write!(f, "Twos"),
            Categories::Threes => write!(f, "Threes"),
            Categories::Fours => write!(f, "Fours"),
            Categories::Fives => write!(f, "Fives"),
            Categories::Sixes => write!(f, "Sixes"),
            Categories::ThreeOfAKind => write!(f, "Three of a Kind"),
            Categories::FourOfAKind => write!(f, "Four of a Kind"),
            Categories::FullHouse => write!(f, "Full House"),
            Categories::SmallStraight => write!(f, "Small Straight"),
            Categories::LargeStraight => write!(f, "Large Straight"),
            Categories::Yahtzee => write!(f, "Yahtzee!"),
            Categories::Chance => write!(f, "Chance"),
        }
    }
}

/// A Yahtzee scorecard, with categories and corresponding scores - if any.
#[derive(Debug, Clone)]
pub struct YahtzeeScorecard {
    pub scores: HashMap<Categories, Option<u8>>,
    pub upper_subtotal: usize, // total of 1-6 before bonus above
    pub upper_bonus: usize,    // 35 if the upper subtotal (1-6es) totals 63 or higher
    pub upper_total: usize,    // final upper score (subtotal + bonus)
    pub extra_yahtzees: usize, // 100 points for every yahtzee beyond the first
    pub lower_total: usize,    // lower scores + bonus yahtzees
    pub final_score: usize,    // upper + lower totals
}

impl YahtzeeScorecard {
    /// Creates a new, blank scorecard.
    pub fn new() -> Self {
        let scores = HashMap::from([
            (Categories::Aces, None),
            (Categories::Twos, None),
            (Categories::Threes, None),
            (Categories::Fours, None),
            (Categories::Fives, None),
            (Categories::Sixes, None),
            (Categories::ThreeOfAKind, None),
            (Categories::FourOfAKind, None),
            (Categories::FullHouse, None),
            (Categories::SmallStraight, None),
            (Categories::LargeStraight, None),
            (Categories::Yahtzee, None),
            (Categories::Chance, None),
        ]);
        Self {
            scores,
            upper_bonus: 0,
            upper_total: 0,
            extra_yahtzees: 0,
            lower_total: 0,
            final_score: 0,
            upper_subtotal: 0,
        }
    }
    /// Record a score on the scorecard.
    pub fn record_score(&mut self, which: Categories, points: usize) -> Result<(), &'static str> {
        if which == Categories::Yahtzee && self.scores[&Categories::Yahtzee].is_some() {
            // got a Yahtzee but already scored one -- goes into the bonus column
            self.extra_yahtzees += 1;
            return Ok(());
        }
        self.scores.insert(which, Some(points as u8));
        Ok(())
    }
}
impl Default for YahtzeeScorecard {
    /// Creates a new blank scorecard.
    fn default() -> Self {
        YahtzeeScorecard::new()
    }
}
impl std::fmt::Display for YahtzeeScorecard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:﹏^30}", "YAHTZEE Scorecard")?;
        writeln!(f, "[UPPER Section]")?;

        for cat in [
            Categories::Aces,
            Categories::Twos,
            Categories::Threes,
            Categories::Fours,
            Categories::Fives,
            Categories::Sixes,
        ] {
            let label = format!("{}", cat);
            let score = self.scores.get(&cat).and_then(|x| *x);
            let display_val = match score {
                Some(score) => score.to_string(),
                None => "〷".to_string(),
            };
            writeln!(f, "{:>15} 🠢 {:>3}", label, display_val)?;
        }
        writeln!(f, "{:>15} 🠢 {:>3}", "Over 62 BONUS", self.upper_bonus)?;
        writeln!(f, "\n[LOWER Section]")?;
        for cat in [
            Categories::ThreeOfAKind,
            Categories::FourOfAKind,
            Categories::FullHouse,
            Categories::SmallStraight,
            Categories::LargeStraight,
            Categories::Yahtzee,
            Categories::Chance,
        ] {
            let label = format!("{}", cat);
            let score = self.scores.get(&cat).and_then(|x| *x);
            let display_val = match score {
                Some(score) => score.to_string(),
                None => "〷".to_string(),
            };
            writeln!(f, "{:>15} 🠢 {:>3}", label, display_val)?;
        }
        let xtra_yahtzee_bonus = 100 * self.extra_yahtzees;
        writeln!(f, "{:>15} 🠢 {:>3}", "EXTRA Yahtzees", self.extra_yahtzees)?;
        writeln!(f, "{:>15} 🠢 {:>3}", "Yahtzee BONUS", xtra_yahtzee_bonus)?;

        writeln!(f, "\n[TOTALS]")?;
        writeln!(f, "UPPER Total: {:>3}", self.upper_total)?;
        writeln!(f, "LOWER Total: {:>3}", self.lower_total)?;
        writeln!(f, "𝄗𝄗𝄗𝄗𝄗𝄗𝄗𝄗𝄗𝄗𝄗𝄗𝄗")?;
        writeln!(f, "GRAND Total: {:>3}", self.final_score)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::yahtzee::*;

    #[test]
    fn can_create_scorecard() {
        let card = YahtzeeScorecard::new();
        assert_eq!(card.scores.len(), 13);
    }

    #[test]
    fn can_create_new_game() {
        let game = GameState::new_game();
        assert_eq!(game.turns_left, 13);
        assert_eq!(game.rolls_left, 3);
        assert_eq!(game.scorecard.scores.len(), 13);
    }

    #[test]
    fn open_categories_returns_all_open_categories() {
        let game = GameState::new_game();
        let open_cats = game.open_categories();
        for category in [
            Categories::Aces,
            Categories::Twos,
            Categories::Threes,
            Categories::Fours,
            Categories::Fives,
            Categories::Sixes,
            Categories::ThreeOfAKind,
            Categories::FourOfAKind,
            Categories::FullHouse,
            Categories::SmallStraight,
            Categories::LargeStraight,
            Categories::Yahtzee,
            Categories::Chance,
        ] {
            assert!(open_cats.contains(&category));
        }
    }

    #[test]
    fn open_categories_only_returns_open_categories() {
        let mut game = GameState::new_game();
        game.scorecard.scores.insert(Categories::Aces, Some(3));
        let open_cats = game.open_categories();
        assert!(!open_cats.contains(&Categories::Aces));
        assert!(open_cats.contains(&Categories::Twos));
    }

    #[test]
    fn open_categories_only_returns_open_categories_with_multiple_scores() {
        let mut game = GameState::new_game();
        game.scorecard.scores.insert(Categories::Aces, Some(3));
        game.scorecard.scores.insert(Categories::Twos, Some(4));
        let open_cats = game.open_categories();
        assert!(!open_cats.contains(&Categories::Aces));
        assert!(!open_cats.contains(&Categories::Twos));
        assert!(open_cats.contains(&Categories::Threes));
    }

    #[test]
    fn open_categories_always_returns_with_yahtzee_open() {
        let mut game = GameState::new_game();
        game.scorecard.scores.insert(Categories::Yahtzee, Some(50));
        assert_eq!(
            game.scorecard.scores.get(&Categories::Yahtzee).unwrap(),
            &Some(50)
        );
        let open_cats = game.open_categories();
        assert!(open_cats.contains(&Categories::Yahtzee));
    }

    #[test]
    fn start_turn_returns_err_if_no_turns_left() {
        let mut game = GameState::new_game();
        game.turns_left = 0;
        assert!(game.start_turn().is_err());
    }

    #[test]
    fn start_turn_returns_err_if_turn_already_in_progress() {
        let mut game = GameState::new_game();
        game.start_turn()
            .expect("should be able to start a turn with brand new game");
        // trying to start again before 3rd roll should cause an error
        assert!(game.start_turn().is_err());
    }

    #[test]
    fn start_turn_creates_correct_dice_pool_5d6() {
        let mut game = GameState::new_game();
        game.start_turn().unwrap();
        assert_eq!(game.dice.size(), 5);
        for &roll in game.dice.results() {
            assert!((1..=6).contains(&roll));
        }
    }

    #[test]
    fn start_turn_decrements_turn_counter() {
        let mut game = GameState::new_game();
        assert_eq!(game.turns_left, 13);
        game.start_turn().unwrap();
        assert_eq!(game.turns_left, 12);
    }

    #[test]
    fn start_turn_decrements_roll_counter() {
        let mut game = GameState::new_game();
        assert_eq!(game.rolls_left, 3);
        game.start_turn().unwrap();
        assert_eq!(game.rolls_left, 2);
    }

    #[test]
    fn reroll_dice_returns_err_if_no_rolls_left_in_turn() {
        let mut game = GameState::new_game();
        game.rolls_left = 0;
        assert!(game.reroll_dice(&[]).is_err());
    }

    #[test]
    fn reroll_dice_decrements_roll_counter() {
        let mut game = GameState::new_game();
        assert_eq!(game.rolls_left, 3);
        game.start_turn().unwrap();
        assert_eq!(game.rolls_left, 2);
        game.reroll_dice(&[]).unwrap();
        assert_eq!(game.rolls_left, 1);
    }

    #[test]
    fn reroll_dice_only_rerolls_specified_dice() {
        let mut changed = [false; 5];
        for _ in 1..20 {
            let mut game = GameState::new_game();
            game.start_turn().unwrap();
            let orig_state = game.clone();
            game.reroll_dice(&[1, 2, 4]).unwrap();
            let new_state = game.clone();
            for i in 0..5 {
                if orig_state.dice.results()[i] != new_state.dice.results()[i] {
                    changed[i] = true;
                }
            }
        }
        assert_eq!(changed, [false, true, true, false, true]);
    }

    #[test]
    fn score_roll_as_returns_correct_scores() {
        let game = GameState::new_game();
        assert_eq!(game.score_roll_as(&[1, 1, 1, 2, 3], Categories::Aces), 3);
        assert_eq!(game.score_roll_as(&[1, 1, 1, 2, 3], Categories::Twos), 2);
        assert_eq!(game.score_roll_as(&[1, 1, 1, 2, 3], Categories::Threes), 3);
        assert_eq!(game.score_roll_as(&[1, 1, 1, 4, 4], Categories::Fours), 8);
        assert_eq!(game.score_roll_as(&[5, 5, 5, 6, 6], Categories::Fives), 15);
        assert_eq!(game.score_roll_as(&[5, 5, 5, 6, 6], Categories::Sixes), 12);
        assert_eq!(
            game.score_roll_as(&[5, 5, 6, 5, 6], Categories::ThreeOfAKind),
            27
        );
        assert_eq!(
            game.score_roll_as(&[1, 1, 1, 2, 1], Categories::FourOfAKind),
            6
        );
        assert_eq!(
            game.score_roll_as(&[1, 6, 1, 6, 6], Categories::FullHouse),
            25
        );
        assert_eq!(
            game.score_roll_as(&[4, 1, 1, 2, 3], Categories::SmallStraight),
            30
        );
        assert_eq!(
            game.score_roll_as(&[4, 5, 1, 2, 3], Categories::LargeStraight),
            40
        );
        assert_eq!(
            game.score_roll_as(&[1, 1, 1, 1, 1], Categories::Yahtzee),
            50
        );
        assert_eq!(game.score_roll_as(&[1, 1, 1, 2, 3], Categories::Chance), 8);
    }
}
