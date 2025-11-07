//! Cards module testing / examples

use gametools::cards::uno_cards::full_uno_set;
use gametools::cards::Deck;
use gametools::TakeCard;

fn main() {
    let mut uno_deck = Deck::new_from_faces("Uno", &full_uno_set());
    uno_deck.shuffle();
    let mut some_cards = uno_deck.take_cards(15);
    some_cards.sort_by(|a, b| a.faces.color.cmp(&b.faces.color));
    for card in some_cards {
        println!("{card}");
    }
}
