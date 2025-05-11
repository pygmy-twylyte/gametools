mod yahtzee;
mod yahtzee_agent;

use yahtzee::GameState;
use yahtzee_agent::YahtzeeAgent;

fn main() {
    let mut game = GameState::new_game();
    let agent = YahtzeeAgent;

    for turn in 1..=13 {
        println!("\n--------- Turn {turn} ---------");
        println!("Open categories: {:?}", game.open_categories());
        game.rolls_left = 3;
        match game.start_turn() {
            Ok(_) => {
                println!("🎲 Starting roll: {:?}", game.dice.results());
                if let Some((cat, score)) = agent.best_scoring_option(&game) {
                    println!(
                        "   🤖 YahtzAI: current game dice are worth {} points ({})",
                        score, cat
                    );
                };

                for roll_number in 2..=3 {
                    if let Some(hold) = agent.choose_hold(&game) {
                        println!(
                            "   🤖 YahtzAI: I calculate a likely best outcome from holding dice: {:?}",
                            hold.iter().map(|x| x + 1).collect::<Vec<usize>>()
                        );
                        let reroll = (0..5).filter(|i| !hold.contains(i)).collect::<Vec<_>>();
                        println!(
                            "   🔄 Rerolling dice: {:?}",
                            reroll.iter().map(|x| x + 1).collect::<Vec<_>>()
                        );

                        game.reroll_dice(&reroll).expect("error on dice re-roll");
                        println!("🎲 After reroll: {:?}", game.dice.results());
                        if let Some((cat, score)) = agent.best_scoring_option(&game) {
                            println!(
                                "   🤖 YahtzAI: dice are now worth {} points (as {}) after roll # {}",
                                score, cat, roll_number
                            );
                        };
                    } else {
                        println!("⚠️  No reroll chosen or not allowed.");
                    }
                }
            }
            Err(e) => {
                println!("🚫 Could not start turn: {}", e);
            }
        }
        if let Some((category, score)) = agent.best_scoring_option(&game) {
            game.scorecard
                .record_score(category, score)
                .unwrap_or_else(|_| println!("couldn't record score"));
        } else {
            println!("🚫 Could not determine a best scoring option");
        }
        println!("✅ End of turn.");
    }
    game.show_final_score();
}
