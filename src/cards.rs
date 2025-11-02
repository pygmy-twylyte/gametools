//! # Cards Module
//!
//! This `gametools` module provides helpful tools for working with cards of
//! any type. `Card` is generic over T where T implements the `CardFaces` trait,
//! which defines how the front and back faces of the card are represented and
//! how/if they can be matched and compared.
//!
//! Unlike the prior implementation, this makes this version of the `cards` module useful
//! for any kind of card: playing cards, Uno cards, Tarot, MAGIC, flashcards, etc.
//!
//!
pub mod card;
pub mod deck;
pub mod hand;
pub mod pile;
pub mod std_playing_cards;

pub use card::{Card, CardFaces};
pub use deck::Deck;
pub use hand::{Hand, Hand as CardHand};
pub use pile::Pile;
pub use std_playing_cards::{Rank, StandardCard, Suit};

/// Methods common to all the different types of card collections
pub trait CardCollection {
    /// Determine the number of cards in the collection.
    fn size(&self) -> usize;
    /// Turn all of the cards face-up / front showing.
    fn show_faces(&mut self);
    /// Turn all of the cards face-down / back showing.
    fn show_backs(&mut self);
}

/// Methods shared by card collections that allow cards to be added to them.
pub trait AddCard<T: CardFaces> {
    /// Add one card to the collection.
    fn add_card(&mut self, card: Card<T>);
    /// Add a list of cards to the collection.
    fn add_cards(&mut self, cards: Vec<Card<T>>) {
        for card in cards {
            self.add_card(card);
        }
    }
}

/// Methods shared by card collections that allow cards to be removed.
pub trait TakeCard<T: CardFaces> {
    /// Take a `Card` (typically the top or last added) from a collection.
    fn take_card(&mut self) -> Option<Card<T>>;
    /// Take `count` `Cards` from a collection.
    ///
    /// May return fewer than requested if the source collection runs dry or
    /// is already empty.
    fn take_cards(&mut self, count: usize) -> Vec<Card<T>> {
        let mut cards = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(card) = self.take_card() {
                cards.push(card);
            } else {
                break;
            }
        }
        cards
    }
    /// Take the `Card` matching the `search_card` from the collection, if it exists.
    fn take_match(&mut self, search_card: &Card<T>) -> Option<Card<T>>;
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

    #[derive(Debug, Default)]
    struct StubCollection {
        cards: Vec<Card<StubFaces>>,
    }

    impl AddCard<StubFaces> for StubCollection {
        fn add_card(&mut self, card: Card<StubFaces>) {
            self.cards.push(card);
        }
    }

    impl TakeCard<StubFaces> for StubCollection {
        fn take_card(&mut self) -> Option<Card<StubFaces>> {
            self.cards.pop()
        }

        fn take_match(&mut self, search_card: &Card<StubFaces>) -> Option<Card<StubFaces>> {
            let idx = self
                .cards
                .iter()
                .position(|card| card.faces.matches(&search_card.faces));
            idx.map(|i| self.cards.remove(i))
        }
    }

    fn make_card(id: u8) -> Card<StubFaces> {
        Card::new_card(StubFaces { id })
    }

    #[test]
    fn add_cards_adds_each_card_in_order() {
        let mut collection = StubCollection::default();
        let cards = vec![make_card(1), make_card(2), make_card(3)];

        collection.add_cards(cards.clone());

        assert_eq!(collection.cards.len(), 3);
        let ids: Vec<u8> = collection.cards.iter().map(|card| card.faces.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn take_cards_respects_count_and_order() {
        let mut collection = StubCollection::default();
        collection.cards = vec![make_card(1), make_card(2), make_card(3)];

        let taken = collection.take_cards(2);

        assert_eq!(taken.len(), 2);
        assert_eq!(taken[0].faces.id, 3);
        assert_eq!(taken[1].faces.id, 2);
        assert_eq!(collection.cards.len(), 1);
        assert_eq!(collection.cards[0].faces.id, 1);
    }

    #[test]
    fn take_cards_stops_when_collection_is_empty() {
        let mut collection = StubCollection::default();
        collection.cards = vec![make_card(5)];

        let taken = collection.take_cards(3);

        assert_eq!(taken.len(), 1);
        assert!(collection.cards.is_empty());
    }
}
