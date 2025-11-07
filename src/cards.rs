//! # Cards Toolkit
//!
//! Utilities for building card-driven games. The module stays agnostic about the
//! artwork or values that appear on the face of each card: any type that implements
//! [`CardFaces`] can be wrapped in a [`Card`] and managed by the provided collections.
//!
//! The building blocks are intentionally composable:
//! * [`Deck`] starts full and only removes cards through draw-like operations.
//! * [`Pile`] starts empty, accepts arbitrary cards, and can be used for discard or staging.
//! * [`Hand`] tracks cards held by a particular player.
//! * [`CardCollection`], [`AddCard`], and [`TakeCard`] are shared traits that let you write
//!   collection-agnostic helper functions.
//!
//! # Examples
//!
//! ## Define a custom card face type and create a deck
//! ```
//! use gametools::{Card, CardFaces, CardCollection, Deck};
//!
//! #[derive(Clone)]
//! struct NumberFace(u8);
//!
//! impl CardFaces for NumberFace {
//!     fn display_front(&self) -> String {
//!         format!("Number {}", self.0)
//!     }
//!
//!     fn display_back(&self) -> Option<String> {
//!         Some(String::from("Back"))
//!     }
//!
//!     fn matches(&self, other: &Self) -> bool {
//!         self.0 == other.0
//!     }
//!
//!     fn compare(&self, other: &Self) -> std::cmp::Ordering {
//!         self.0.cmp(&other.0)
//!     }
//! }
//!
//! let cards = (1..=3)
//!     .map(|value| Card::new_card(NumberFace(value)))
//!     .collect::<Vec<_>>();
//!
//! let mut deck = Deck::new("number-demo", cards);
//! assert_eq!(deck.size(), 3);
//! deck.show_backs();
//! ```
//!
//! ## Share helpers across collections
//! ```
//! use gametools::{AddCard, Card, CardCollection, CardFaces, Deck, Hand, TakeCard};
//!
//! #[derive(Clone)]
//! struct Stub(i32);
//!
//! impl CardFaces for Stub {
//!     fn display_front(&self) -> String { format!("{}", self.0) }
//!     fn display_back(&self) -> Option<String> { None }
//!     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
//!     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
//! }
//!
//! fn draw_three<T: CardFaces, C>(collection: &mut C) -> Vec<Card<T>>
//! where
//!     C: TakeCard<T> + CardCollection,
//! {
//!     collection.take_cards(3)
//! }
//!
//! let cards = (0..5).map(|n| Card::new_card(Stub(n))).collect();
//! let mut deck = Deck::new("demo", cards);
//! let hand_cards = draw_three::<Stub, _>(&mut deck);
//!
//! let mut hand = Hand::<Stub>::new("player");
//! hand.add_cards(hand_cards);
//! assert_eq!(hand.size(), 3);
//! ```
pub mod card;
pub mod deck;
pub mod hand;
pub mod pile;
pub mod std_playing_cards;
pub mod uno_cards;

pub use card::{Card, CardFaces};
pub use deck::Deck;
pub use hand::{Hand, Hand as CardHand};
pub use pile::Pile;
pub use std_playing_cards::{Rank, StandardCard, Suit};

/// Shared behaviors for card containers such as [`Deck`], [`Hand`], and [`Pile`].
pub trait CardCollection {
    /// Determine the number of cards in the collection.
    fn size(&self) -> usize;
    /// Turn all of the cards face-up / front showing.
    fn show_faces(&mut self);
    /// Turn all of the cards face-down / back showing.
    fn show_backs(&mut self);
}

/// Shared behavior for card collections that accept new cards.
pub trait AddCard<T: CardFaces> {
    /// Add one card to the collection.
    fn add_card(&mut self, card: Card<T>);
    /// Add a list of cards to the collection.
    ///
    /// ```
    /// use gametools::{AddCard, Card, CardFaces, Hand};
    ///
    /// #[derive(Clone)]
    /// struct Face(u8);
    ///
    /// impl CardFaces for Face {
    ///     fn display_front(&self) -> String { format!("{}", self.0) }
    ///     fn display_back(&self) -> Option<String> { None }
    ///     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
    ///     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
    /// }
    ///
    /// let mut hand = Hand::<Face>::new("player");
    /// hand.add_cards(vec![Card::new_card(Face(1)), Card::new_card(Face(2))]);
    /// assert_eq!(hand.cards.len(), 2);
    /// ```
    fn add_cards(&mut self, cards: Vec<Card<T>>) {
        for card in cards {
            self.add_card(card);
        }
    }
}

/// Shared behavior for card collections that yield cards.
pub trait TakeCard<T: CardFaces> {
    /// Take a `Card` (typically the top or last added) from a collection.
    fn take_card(&mut self) -> Option<Card<T>>;
    /// Take `count` `Cards` from a collection.
    ///
    /// May return fewer than requested if the source collection runs dry or
    /// is already empty.
    ///
    /// ```
    /// use gametools::{Card, CardFaces, Deck, TakeCard};
    ///
    /// #[derive(Clone)]
    /// struct Face(u8);
    ///
    /// impl CardFaces for Face {
    ///     fn display_front(&self) -> String { format!("{}", self.0) }
    ///     fn display_back(&self) -> Option<String> { None }
    ///     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
    ///     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
    /// }
    ///
    /// let mut deck = Deck::new(
    ///     "demo",
    ///     vec![Card::new_card(Face(1)), Card::new_card(Face(2)), Card::new_card(Face(3))],
    /// );
    ///
    /// let drawn = deck.take_cards(2);
    /// assert_eq!(drawn.len(), 2);
    /// assert_eq!(drawn[0].faces.0, 3);
    /// assert_eq!(drawn[1].faces.0, 2);
    /// ```
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
