//! scratchpad
//!
//! this is a space used to hand-test and debug things during development

use gametools::cards::std_playing_cards as stdcard;
use gametools::cards::{Card, CardCollection, CardFaces, Deck, Pile, StandardCard, transfer_card};
use gametools::{AddCard, TakeCard};

fn main() {
    let mut deck = Deck::with_borrowed_faces("Std Deck", &stdcard::euchre_deck());
    let mut hands = deck.deal(&["dave", "chris"], 5);
    let mut hand_1 = hands.pop().unwrap();
    let mut hand_2 = hands.pop().unwrap();

    for ref hand in [&hand_1, &hand_2] {
        print!("{} >> ", hand.player);
        show_cards(hand.cards());
    }

    // lets move the king of spades around -- since we didn't shuffle the cards
    // it will be in hand_1; we'll move it to hand_2
    let mover = StandardCard::new_card(stdcard::Rank::King, stdcard::Suit::Spades);
    match transfer_card(&Card::from(mover), &mut hand_1, &mut hand_2) {
        Ok(_) => println!("Card transfer successful."),
        Err(e) => println!("Transfer failed: {e}"),
    };
    assert!(!hand_1.contains(stdcard::Rank::King, stdcard::Suit::Spades));
    assert!(hand_2.contains(stdcard::Rank::King, stdcard::Suit::Spades));

    for hand in [&hand_1, &hand_2] {
        print!("{} >> ", hand.player);
        show_cards(hand.cards());
    }

    // note: this does not compile because cards cannot be added to a deck (Deck is not AddCard)
    // let _ = transfer_card(&Card::from(mover), &mut hand_2, &mut deck);
    //                                                             ^^^^

    // make a discard pile, and pass the king of spades into it from hand_2
    let mut discard = Pile::new_pile("discard");
    match transfer_card(&mover.into(), &mut hand_2, &mut discard) {
        Ok(_) => println!("Discard from hand 2 successful."),
        Err(e) => println!("Txfer to discard pile failed: {e}"),
    }

    print!("{} >>", hand_2.player);
    show_cards(hand_2.cards());
    print!("{} >>", discard.name);
    show_cards(&discard.cards());

    // confirm the card moved from hand_1 -> hand_2 -> discard is the one from the deck,
    // not a search card somehow added to the mix.
    assert!(&discard.check_top_card().unwrap().is_from_deck(&deck));
    assert_eq!(discard.size(), 1);

    discard.add_card(deck.take_card().unwrap());
}

fn show_cards<T: CardFaces>(cards: &[Card<T>]) {
    for card in cards {
        print!("|{card}| ");
    }
    println!();
}
