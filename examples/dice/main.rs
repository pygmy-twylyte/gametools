//! Dice module examples
//!
//! No unifything theme here -- just demonstrating some likely common uses of the module.

use gametools::{Die, dice::DieResult};

fn main() -> DieResult<()> {
    // Ability score -- roll 4 x d6 and sum the highest 3.
    let d6_rolls = Die::new(6)?.roll_n(4);
    println!("Ability score roll (4d6): {:?}", d6_rolls.as_slice());
    println!("Sum of top three: {}", d6_rolls.highest(3).sum());
    println!();

    // Attacking with 10 squads, but each must have over 15 on a d20 to hit.
    // 1 = critical miss, 20 = critical hit
    let d20_rolls = Die::new(20)?.roll_n(10);
    println!("Attack rolls (>13 to hit): {:?}", d20_rolls.as_slice());
    println!(
        "{}/10 squads hit their targets.",
        d20_rolls.count_where(|r| r > 13)
    );
    println!(
        "{}/10 captured their objectives.",
        d20_rolls.count_where(|r| r == 20)
    );
    println!("{}/10 were wiped out.", d20_rolls.count_where(|r| r == 1));
    println!();

    // Rolls::histogram() can be particularly helpful in games like Yahtzee or
    // Farkle, where rolls are evaluated like poker hands.
    let yahtzee_roll = Die::new(6)?.roll_n(5);
    let histogram = yahtzee_roll.histogram();
    println! {"Yahtzee Roll (5d6): {:?}", yahtzee_roll.as_slice()};
    println! {"Histogram: {:#?}", histogram};
    if histogram.len() == 2 && histogram.values().any(|count| *count == 3) {
        println!("Full House!");
    }
    if histogram.len() == 1 {
        println!("Yahtzee!");
    }
    println!();

    // A damage roll -- 10 exploding d4 (explodes on 4)
    let damage_roll = Die::exploding(4, 4)?.roll_n(10);
    println!("Damage Roll: {:?}", damage_roll.as_slice());
    println!("Total: {}", damage_roll.sum());
    println!();

    Ok(())
}
