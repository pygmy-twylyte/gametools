use gametools::dice::*;
use gametools::cards::*;

fn main() {
    // playground / example dev for gametools library

    let d20 = Die::new(20);
    let dice_pool = d20.roll_into_pool(10);
    
    let roll_count = dice_pool.size();
    let crit_count = dice_pool.count_roll(20);
    let fail_count = dice_pool.count_roll(1);
    let (min_roll, max_roll) = dice_pool.range().unwrap();
    let total = dice_pool.sum();
    
    println!("DicePool stats: {roll_count} dice rolled / {crit_count} crits / {fail_count} fails");
    println!("Lowest roll {min_roll}, highest roll {max_roll}, total of all dice {total}\n");
    println!("Initial pool of d20 rolls:\n\t{:?}", dice_pool);
    
    // reroll any 1's with a new d20
    let ones_rerolled = dice_pool.reroll_if(&d20, |r| r == 1);
    println!("After ones rerolled:\n\t{:?}", ones_rerolled);

    // reroll ones and then take only the top 5 rolls
    let new_pool = dice_pool.reroll_if(&d20, |r| r == 1).take_highest(5);
    println!("Ones re-rerolled and top 5 taken:\n\t{:?}", new_pool);

    // power up! all rolls buffed +3!
    let new_pool = new_pool.buff(3);
    println!("...then all buffed by 3:\n\t{:?}", new_pool);

    // exploding dice! rolling max allows you to keep rolling
    let d6 = Die::new(6);
    let mut exploders = vec![];
    for _ in 1..15 {
        exploders.push(d6.roll_exploding());
    }
    exploders.sort_by(|a, b| b.cmp(a));
    println!("Some rolls from an 'exploding' d6:\n\t{:?}", exploders);

    // create and roll a pool of ten d10's and count success rolls of 8 or higher
    let wins = Die::new(10).roll_into_pool(10).count_success_over(7);
    println!("Success (8+) on {wins}/10 d10 rolls.");

    // same thing, but this time player has a bonus +1 to all rolls
    let wins = Die::new(10).roll_into_pool(10).buff(1).count_success_over(7);
    println!("Success on another {wins}/10 d10 rolls after +1 buff.");

    ///////////////////////////////////// cards section /////////////////////////////////////////
    
    let mut deck = Deck::new("Playing Cards");
    deck.shuffle();
    if let Some(hand) = deck.draw_cards(5) {
        for card in &hand {
            println!("{card}");
        }
    };


}
