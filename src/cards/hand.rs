//! # Hands
//!
//! A [`Hand`] represents the cards held by a single player. Hands implement
//! [`CardCollection`], [`AddCard`], and [`TakeCard`] so they slot into the same
//! helper routines as decks and piles.
//!
//! ```
//! use gametools::{AddCard, Card, CardCollection, CardFaces, Hand, TakeCard};
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
//! let mut hand = Hand::<Face>::new("alice");
//! hand.add_card(Card::new_card(Face(7)));
//! hand.add_card(Card::new_card(Face(3)));
//! assert_eq!(hand.size(), 2);
//! let top = hand.take_card().unwrap();
//! assert_eq!(top.faces.0, 3);
//! ```
use crate::cards::{AddCard, Card, CardCollection, CardFaces, TakeCard};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Cards held by a single player.
pub struct Hand<T: CardFaces> {
    /// Player identifier or display name.
    pub player: String,
    /// Cards currently in the player's hand. The last card is considered the "top".
   pub cards: Vec<Card<T>>,
}

impl<T: CardFaces> CardCollection for Hand<T> {
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

impl<T: CardFaces> Hand<T> {
    /// Create an empty hand for the supplied `player`.
    ///
    /// ```
    /// use gametools::{CardFaces, Hand};
    ///
    /// #[derive(Clone)]
    /// struct Face;
    ///
    /// impl CardFaces for Face {
    ///     fn display_front(&self) -> String { String::from("X") }
    ///     fn display_back(&self) -> Option<String> { None }
    ///     fn matches(&self, _other: &Self) -> bool { true }
    ///     fn compare(&self, _other: &Self) -> std::cmp::Ordering {
    ///         std::cmp::Ordering::Equal
    ///     }
    /// }
    ///
    /// let hand = Hand::<Face>::new("player");
    /// assert_eq!(hand.player, "player");
    /// assert_eq!(hand.cards.len(), 0);
    /// ```
    pub fn new(player: &str) -> Self {
        Self {
            player: player.to_string(),
            cards: Vec::<Card<T>>::new(),
        }
    }
}

impl<T: CardFaces> AddCard<T> for Hand<T> {
    /// Add a card to the end (top) of the hand.
    fn add_card(&mut self, card: Card<T>) {
        self.cards.push(card);
    }
}

impl<T: CardFaces> TakeCard<T> for Hand<T> {
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
    fn new_creates_empty_hand_for_player() {
        let hand = Hand::<StubFaces>::new("bob");

        assert_eq!(hand.player, "bob");
        assert!(hand.cards.is_empty());
    }

    #[test]
    fn add_card_pushes_to_hand() {
        let mut hand = Hand::<StubFaces>::new("bob");
        hand.add_card(make_card(1));

        assert_eq!(hand.cards.len(), 1);
        assert_eq!(hand.cards[0].faces.id, 1);
    }

    #[test]
    fn take_card_returns_last_card_added() {
        let mut hand = Hand::<StubFaces>::new("bob");
        hand.add_card(make_card(1));
        hand.add_card(make_card(2));

        assert_eq!(hand.take_card().unwrap().faces.id, 2);
        assert_eq!(hand.take_card().unwrap().faces.id, 1);
        assert!(hand.take_card().is_none());
    }

    #[test]
    fn take_match_removes_matching_card() {
        let mut hand = Hand::<StubFaces>::new("bob");
        hand.add_card(make_card(1));
        hand.add_card(make_card(2));
        hand.add_card(make_card(3));
        let search = Card::new_card(StubFaces { id: 2 });

        let taken = hand.take_match(&search).expect("card should be removed");

        assert_eq!(taken.faces.id, 2);
        let ids: Vec<u8> = hand.cards.iter().map(|c| c.faces.id).collect();
        assert_eq!(ids, vec![1, 3]);
    }
}
