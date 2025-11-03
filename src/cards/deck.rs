//! deck module
//!
//! Represents a `Deck` of `Card<T>`
//! `Deck` differs from `Pile` in that a `Deck` begins full and only be drawn from.
//! `Pile` starts empty and cards can be added to or taken from it.
//!
//! Implements: Debug, Clone, PartialEq, CardCollection, TakeCard

use crate::cards::{AddCard, Card, CardCollection, CardFaces, Hand, TakeCard};
use rand::prelude::*;
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeckId(Uuid);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Deck<T: CardFaces> {
    pub name: String,
    pub deck_id: DeckId,
    pub cards: Vec<Card<T>>,
}
impl<T: CardFaces + Clone> Deck<T> {
    pub fn new(name: &str, cards: &mut Vec<Card<T>>) -> Self {
        let deck_id = DeckId(Uuid::new_v4());
        cards.iter_mut().for_each(|c| c.assign_to_deck(deck_id));
        Self {
            name: name.to_string(),
            deck_id,
            cards: cards.to_vec(),
        }
    }

    pub fn new_from_faces(name: &str, faces: &Vec<T>) -> Self {
        let mut cards = faces
            .iter()
            .map(|f| Card::new_card(f.clone()))
            .collect::<Vec<_>>();
        Self::new(name, &mut cards)
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::rng());
    }

    /// Determine whether the supplied `Card` belongs to this `Deck`.
    pub fn owns_card(&self, card: &Card<T>) -> bool {
        if let Some(card_deck_id) = &card.deck_id {
            self.deck_id == *card_deck_id
        } else {
            false
        }
    }

    pub fn deal(&mut self, players: &[&str], count: usize) -> Vec<Hand<T>> {
        // create hands for the players
        let mut hands: Vec<Hand<T>> = players.iter().map(|name| Hand::<T>::new(name)).collect();
        // deal `count` cards to each `Hand`
        for _ in 0..count {
            for hand in &mut hands {
                if let Some(card) = self.take_card() {
                    hand.add_card(card);
                }
            }
        }
        // return the `Hand` list
        hands
    }
}
impl<T: CardFaces> CardCollection for Deck<T> {
    fn size(&self) -> usize {
        self.cards.len()
    }

    fn show_faces(&mut self) {
        for ref mut card in &mut self.cards {
            card.face_up = true;
        }
    }

    fn show_backs(&mut self) {
        for ref mut card in &mut self.cards {
            card.face_up = false;
        }
    }
}
impl<T: CardFaces> TakeCard<T> for Deck<T> {
    fn take_card(&mut self) -> Option<Card<T>> {
        self.cards.pop()
    }

    fn take_match(&mut self, search_card: &Card<T>) -> Option<Card<T>> {
        let idx = self
            .cards
            .iter()
            .position(|c| c.faces.matches(&search_card.faces));
        if let Some(i) = idx {
            Some(self.cards.remove(i))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct StubFaces {
        id: u8,
    }

    impl CardFaces for StubFaces {
        fn display_front(&self) -> String {
            format!("front-{}", self.id)
        }

        fn display_back(&self) -> Option<String> {
            None
        }

        fn matches(&self, other: &Self) -> bool {
            self.id == other.id
        }

        fn compare(&self, other: &Self) -> std::cmp::Ordering {
            self.id.cmp(&other.id)
        }
    }

    fn make_card(id: u8) -> Card<StubFaces> {
        Card::new_card(StubFaces { id })
    }

    #[test]
    fn new_from_faces_builds_deck_with_expected_cards() {
        let faces = vec![StubFaces { id: 1 }, StubFaces { id: 2 }];

        let deck = Deck::new_from_faces("test", &faces);

        assert_eq!(deck.name, "test");
        let ids: Vec<u8> = deck.cards.iter().map(|c| c.faces.id).collect();
        assert_eq!(ids, vec![1, 2]);
        // ensure original faces untouched
        assert_eq!(faces[0].id, 1);
    }

    #[test]
    fn take_card_removes_last_card() {
        let mut deck = Deck::new("test", vec![make_card(1), make_card(2)]);

        let taken = deck.take_card().unwrap();

        assert_eq!(taken.faces.id, 2);
        assert_eq!(deck.cards.len(), 1);
    }

    #[test]
    fn take_match_removes_matching_card() {
        let mut deck = Deck::new("test", vec![make_card(1), make_card(2), make_card(3)]);
        let search = Card::new_card(StubFaces { id: 2 });

        let taken = deck.take_match(&search).expect("card should be found");

        assert_eq!(taken.faces.id, 2);
        let remaining: Vec<u8> = deck.cards.iter().map(|c| c.faces.id).collect();
        assert_eq!(remaining, vec![1, 3]);
    }

    #[test]
    fn deal_distributes_cards_round_robin() {
        let cards = vec![
            make_card(1),
            make_card(2),
            make_card(3),
            make_card(4),
            make_card(5),
            make_card(6),
        ];
        let mut deck = Deck::new("test", cards);
        let players = vec!["alice", "bob"];

        let hands = deck.deal(&players, 2);

        assert_eq!(hands.len(), 2);
        assert_eq!(hands[0].player, "alice");
        assert_eq!(hands[1].player, "bob");
        let alice_ids: Vec<u8> = hands[0].cards.iter().map(|c| c.faces.id).collect();
        let bob_ids: Vec<u8> = hands[1].cards.iter().map(|c| c.faces.id).collect();
        assert_eq!(alice_ids, vec![6, 4]);
        assert_eq!(bob_ids, vec![5, 3]);
        let deck_ids: Vec<u8> = deck.cards.iter().map(|c| c.faces.id).collect();
        assert_eq!(deck_ids, vec![1, 2]);
    }

    #[test]
    fn deal_gracefully_handles_insufficient_cards() {
        let cards = vec![make_card(1), make_card(2), make_card(3)];
        let mut deck = Deck::new("test", cards);
        let players = vec!["a", "b"];

        let hands = deck.deal(&players, 2);

        let lengths: Vec<usize> = hands.iter().map(|hand| hand.cards.len()).collect();
        assert_eq!(lengths, vec![2, 1]);
        assert!(deck.cards.is_empty());
    }
}
