// use gametools::cards::*;
// use gametools::dice::*;
use gametools::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
   
    /*
    Dominos - first play demo
     */
    
    // Create a new set of dominos, 0-12 pips on each side
    let mut pile = BonePile::new(12);
    // Draw 15 of those into a hand
    let mut hand = DominoHand::new_with_draw("me", 15, &mut pile)?;
    // First round starts from 12 (note 12|12 would be removed from player who had it in real game)
    let start_from = 12;
    // Find the longest starting line that can be played on start_from with this hand
    let best_line_ids = hand.find_longest_from(start_from);
    // Play that best line on the player's train
    let mut p1_train = Train::new("david", false, start_from);
    hand.play_line(&best_line_ids, &mut p1_train);
    println!("{hand}");
    println!("{p1_train}");

    Ok(())
}