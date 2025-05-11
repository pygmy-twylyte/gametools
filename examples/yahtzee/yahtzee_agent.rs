use crate::yahtzee::{Categories, GameState};
/// An "intelligent" agent to aid in Yahtzee game decisions
#[derive(Debug)]
pub struct YahtzeeAgent;

impl YahtzeeAgent {
    /// Given a game state, will return the best available scoring option. Returns None
    /// if there are no more open categories.
    pub fn best_scoring_option(&self, game: &GameState) -> Option<(Categories, usize)> {
        let open_cats = game.open_categories();
        if open_cats.is_empty() {
            return None;
        }
        let mut best_cat = *open_cats.first().expect("open_cats should not be empty");
        let mut best_score = game.score_roll_as(&[], best_cat);

        for category in open_cats {
            let points = game.score_roll_as(&[], category);
            if points > best_score {
                best_score = points;
                best_cat = category;
            }
        }
        Some((best_cat, best_score))
    }

    /// Evaluates the current pool of dice to determine which to hold and which to re-roll.
    /// We currently to this by generating all possible combinations of held/re-rolled dice
    /// and all possible outcomes from those rerolls, determining which "hold" leads to the
    /// greatest expected (average) point value after the re-roll.
    pub fn choose_hold(&self, state: &GameState) -> Option<Vec<usize>> {
        let current_roll = state.dice.results(); // &[u8]
        let open_cats = state.open_categories();
        let rolls_left = state.rolls_left;

        if rolls_left == 0 {
            return None; // no rerolls left
        }

        // generate all possible combinations of held / re-rolled dice
        let all_holds = generate_all_holds();

        all_holds
            .into_iter()
            .map(|held_indices| {
                // simulate all possible outcomes for each hold/re-roll combination
                let simulated_rolls = simulate_hold(current_roll, &held_indices);
                // add up all the best possible scores (using only the remaining open categories)
                // for each possible re-roll outcome
                let total_score: usize = simulated_rolls
                    .iter()
                    .map(|roll| best_possible_score(roll, state, &open_cats))
                    .sum();
                // calculate an average, "expected" score (after 1 re-roll) for each hold/re-roll combo
                let avg_score = total_score as f32 / simulated_rolls.len() as f32;
                (held_indices, avg_score)
            })
            // find the hold that has the highest expected return
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            // return that best hold pattern (strip off expected score)
            .map(|(best_hold, _)| best_hold)
    }
}

/// A vector holding a list of dice to be
pub type DiceHeld = Vec<usize>;

fn generate_all_holds() -> Vec<DiceHeld> {
    // There are 32 (2^5) possible combinations of held/rerolled dice
    // These go from 0 (0b00000) for no dice held to 31 (0b11111) for all held.
    (0..32)
        .map(|mask| {
            (0..5) // the we shift bitwise through the mask to determine if an index is included (1) or not (0)
                .filter(|die_idx| (mask >> die_idx) & 1 == 1)
                .collect()
        })
        .collect()
}

fn simulate_hold(roll: &[u8], hold: &[usize]) -> Vec<[u8; 5]> {
    let mut results = Vec::new();
    let reroll_count = 5 - hold.len(); // # of dice that need to be rerolled recursively
    let mut buffer = vec![0u8; reroll_count];

    fn recurse(
        results: &mut Vec<[u8; 5]>,
        buffer: &mut [u8],
        pos: usize,
        roll: &[u8],
        hold: &[usize],
    ) {
        if pos == buffer.len() {
            let mut full = [0u8; 5];
            let mut buffer_idx = 0;
            for i in 0..5 {
                if hold.contains(&i) {
                    full[i] = roll[i];
                } else {
                    full[i] = buffer[buffer_idx];
                    buffer_idx += 1;
                }
            }
            results.push(full);
        } else {
            for val in 1..=6 {
                buffer[pos] = val;
                recurse(results, buffer, pos + 1, roll, hold);
            }
        }
    }

    recurse(&mut results, &mut buffer, 0, roll, hold);
    results
}

fn best_possible_score(roll: &[u8], game: &GameState, cats: &[Categories]) -> usize {
    cats.iter()
        .map(|cat| game.score_roll_as(roll, *cat))
        .max()
        .unwrap_or(0)
}
