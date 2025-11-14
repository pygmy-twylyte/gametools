//! Hybrid demo showcasing the `cards` module with standard playing cards and Uno cards.

use std::cmp::Ordering;

use gametools::{
    Card, CardFaces, Deck, Hand, TakeCard,
    cards::{
        std_playing_cards::{self, StandardCard, Suit},
        uno_cards::{self, UnoCard, UnoCardKind, UnoColor},
    },
};

fn main() {
    println!("=== Card Night Mashup ===");
    let showdown = run_high_card_showdown();
    println!();
    run_uno_color_rally(&showdown);
}

struct ShowdownResult {
    declared_color: UnoColor,
    highlight_card: StandardCard,
    highlight_owner: &'static str,
    champion: Option<&'static str>,
}

fn run_high_card_showdown() -> ShowdownResult {
    const PLAYERS: [&str; 2] = ["Ava", "Bo"];

    let faces = std_playing_cards::standard_52();
    let mut deck = Deck::with_borrowed_faces("classic-52", &faces);
    deck.shuffle();

    let mut hands = deck.deal(&PLAYERS, 3);
    let mut bo_hand = hands.pop().expect("hand for Bo");
    let mut ava_hand = hands.pop().expect("hand for Ava");

    let mut scores = [0_u8, 0_u8];
    let mut spotlight: Option<(StandardCard, &'static str)> = None;

    println!("High-card duel! Each player reveals three cards; best card chooses the Uno color.");
    for round in 1..=3 {
        let ava_card = ava_hand
            .take_card()
            .expect("Ava should still have a card for the round");
        let bo_card = bo_hand
            .take_card()
            .expect("Bo should still have a card for the round");

        let cmp = ava_card.faces.compare(&bo_card.faces);
        let verdict = match cmp {
            Ordering::Greater => {
                scores[0] += 1;
                update_spotlight(&mut spotlight, ava_card.faces, PLAYERS[0]);
                "Ava takes it!"
            }
            Ordering::Less => {
                scores[1] += 1;
                update_spotlight(&mut spotlight, bo_card.faces, PLAYERS[1]);
                "Bo snags the point!"
            }
            Ordering::Equal => {
                update_spotlight(&mut spotlight, ava_card.faces, "Both");
                "Perfect tie."
            }
        };

        println!("Round {round}: Ava shows {ava_card}, Bo shows {bo_card} — {verdict}");
    }

    let champion = match scores[0].cmp(&scores[1]) {
        Ordering::Greater => Some(PLAYERS[0]),
        Ordering::Less => Some(PLAYERS[1]),
        Ordering::Equal => None,
    };

    let (highlight_card, highlight_owner) = spotlight.expect("at least one card should exist");
    let declared_color = suit_to_uno_color(highlight_card.suit);

    println!(
        "Scores -> Ava: {}, Bo: {}. {}",
        scores[0],
        scores[1],
        match champion {
            Some(name) => format!("{name} wins the duel!"),
            None => "They tie, so bragging rights are delayed.".to_string(),
        }
    );
    println!(
        "Flashiest card: {} (played by {}). That maps to the Uno color {:?}.",
        describe_standard_card(highlight_card),
        highlight_owner,
        declared_color
    );

    ShowdownResult {
        declared_color,
        highlight_card,
        highlight_owner,
        champion,
    }
}

fn run_uno_color_rally(showdown: &ShowdownResult) {
    let faces = uno_cards::full_uno_set();
    let mut deck = Deck::with_borrowed_faces("uno", &faces);
    deck.shuffle();

    let players = ["Ava", "Bo", "Kai"];
    let hands = deck.deal(&players, 7);
    let top_card = deck
        .take_card()
        .expect("discard pile should start with one card");

    println!("--- Uno Color Rally ---");
    println!(
        "{}'s {} crowned {:?} as the lucky color.",
        showdown.highlight_owner,
        describe_standard_card(showdown.highlight_card),
        showdown.declared_color
    );
    println!("Starter discard: {top_card}");
    for hand in &hands {
        print_hand(hand);
    }

    println!("\nPlayable cards with the starter discard:");
    for hand in &hands {
        explain_playable(hand, &top_card, None);
    }

    println!(
        "\nSomeone drops a Wild and declares {:?}. Who can follow it?",
        showdown.declared_color
    );
    let declared_top = Card::new_card(UnoCard {
        color: UnoColor::Black,
        kind: UnoCardKind::Wild,
    });
    for hand in &hands {
        explain_playable(hand, &declared_top, Some(showdown.declared_color));
    }

    if let Some(champion) = showdown.champion {
        println!("\nBonus lore: {champion} also won the high-card duel.");
    } else {
        println!("\nBonus lore: the duel was a tie, so the Wild color was crowd-sourced.");
    }
}

fn update_spotlight(
    spotlight: &mut Option<(StandardCard, &'static str)>,
    candidate: StandardCard,
    owner: &'static str,
) {
    let replace = spotlight
        .map(|(current, _)| candidate > current)
        .unwrap_or(true);
    if replace {
        *spotlight = Some((candidate, owner));
    }
}

fn suit_to_uno_color(suit: Suit) -> UnoColor {
    match suit {
        Suit::Hearts => UnoColor::Red,
        Suit::Diamonds => UnoColor::Yellow,
        Suit::Clubs => UnoColor::Green,
        Suit::Spades => UnoColor::Blue,
        Suit::Wild => UnoColor::Black,
    }
}

fn describe_standard_card(face: StandardCard) -> String {
    format!("{}{}", face.rank, face.suit)
}

fn print_hand<T: CardFaces>(hand: &Hand<T>) {
    println!("{}'s hand:", hand.player);
    for (idx, card) in hand.cards().iter().enumerate() {
        println!("  [{idx}] {card}");
    }
}

fn explain_playable(hand: &Hand<UnoCard>, top: &Card<UnoCard>, declared: Option<UnoColor>) {
    let context = match declared {
        Some(color) => format!("{top} with color {:?} declared", color),
        None => format!("{top}"),
    };
    let playable = hand.playable_on(top, declared);
    if playable.is_empty() {
        println!("  {} has nothing for {context} and must draw.", hand.player);
        return;
    }

    println!("  {} can play on {context} using:", hand.player);
    for (idx, card) in playable {
        println!("    -> slot #{idx}: {card}");
    }
}
