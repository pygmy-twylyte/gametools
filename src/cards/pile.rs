//! # Piles
//!
//! A [`Pile`] is an initially empty collection of cards that you can add to and draw from.
//! Use it for discard piles, staging areas, or any shared pool of cards outside a player's hand.
//!
//! ```
//! use gametools::{AddCard, Card, CardFaces, Pile, TakeCard};
//!
//! #[derive(Clone)]
//! struct Face(u8);
//!
//! impl CardFaces for Face {
//!     fn display_front(&self) -> String { format!("{}", self.0) }
//!     fn display_back(&self) -> Option<String> { None }
//!     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
//!     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
//! }
//!
//! let mut pile = Pile::<Face>::new_pile("discard");
//! pile.add_card(Card::new_card(Face(10)));
//! let top = pile.take_card().unwrap();
//! assert_eq!(top.faces.0, 10);
//! ```
use crate::cards::{AddCard, Card, CardCollection, CardFaces, TakeCard};

use rand::seq::SliceRandom;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A named stack of cards that starts empty.
pub struct Pile<T: CardFaces> {
    /// The descriptive name of the pile (for logging or UI).
    pub name: String,
    /// Cards currently stored in the pile.
    pub cards: Vec<Card<T>>,
}
impl<T: CardFaces> CardCollection for Pile<T> {
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

impl<T: CardFaces> Pile<T> {
    /// Create an empty pile with the supplied name.
    ///
    /// ```
    /// use gametools::{CardFaces, Pile};
    ///
    /// #[derive(Clone)]
    /// struct Face;
    ///
    /// impl CardFaces for Face {
    ///     fn display_front(&self) -> String { String::from("front") }
    ///     fn display_back(&self) -> Option<String> { None }
    ///     fn matches(&self, _other: &Self) -> bool { true }
    ///     fn compare(&self, _other: &Self) -> std::cmp::Ordering {
    ///         std::cmp::Ordering::Equal
    ///     }
    /// }
    ///
    /// let pile = Pile::<Face>::new_pile("discard");
    /// assert_eq!(pile.name, "discard");
    /// assert!(pile.cards.is_empty());
    /// ```
    pub fn new_pile(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cards: Vec::<Card<T>>::new(),
        }
    }
}
impl<T: CardFaces> Pile<T> {
    /// Peek at the card on top of the pile.
    pub fn check_top_card(&self) -> Option<&Card<T>> {
        self.cards.last()
    }
    /// Shuffle the cards in the pile
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }
}

impl<T: CardFaces> AddCard<T> for Pile<T> {
    /// Push a card onto the top of the pile.
    fn add_card(&mut self, card: Card<T>) {
        self.cards.push(card);
    }
}

impl<T: CardFaces> TakeCard<T> for Pile<T> {
    /// Remove and return the most recently added card, if any remain.
    fn take_card(&mut self) -> Option<Card<T>> {
        self.cards.pop()
    }

    /// Remove the first card that matches the provided `search_card`.
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
            Some(format!("back-{}", self.id))
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
    fn new_pile_starts_empty() {
        let pile = Pile::<StubFaces>::new_pile("discard");

        assert_eq!(pile.name, "discard");
        assert!(pile.cards.is_empty());
    }

    #[test]
    fn add_card_appends_to_pile() {
        let mut pile = Pile::<StubFaces>::new_pile("discard");
        pile.add_card(make_card(10));

        assert_eq!(pile.cards.len(), 1);
        assert_eq!(pile.cards[0].faces.id, 10);
    }

    #[test]
    fn take_card_returns_last_card_added() {
        let mut pile = Pile::<StubFaces>::new_pile("discard");
        pile.add_card(make_card(1));
        pile.add_card(make_card(2));

        assert_eq!(pile.take_card().unwrap().faces.id, 2);
        assert_eq!(pile.take_card().unwrap().faces.id, 1);
        assert!(pile.take_card().is_none());
    }

    #[test]
    fn take_match_removes_matching_card() {
        let mut pile = Pile::<StubFaces>::new_pile("discard");
        pile.add_card(make_card(1));
        pile.add_card(make_card(2));
        pile.add_card(make_card(3));
        let search = Card::new_card(StubFaces { id: 2 });

        let taken = pile.take_match(&search).expect("card should exist");

        assert_eq!(taken.faces.id, 2);
        let ids: Vec<u8> = pile.cards.iter().map(|c| c.faces.id).collect();
        assert_eq!(ids, vec![1, 3]);
    }
}
