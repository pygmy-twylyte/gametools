//! # Card Primitives
//!
//! Types and traits for representing the smallest unit in the cards ecosystem: an
//! individual card with arbitrary faces. A [`Card`] simply wraps your custom face type
//! and records metadata such as a UUID, deck membership, and whether the card is face up.
//!
//! To plug into the rest of the module you only need to implement [`CardFaces`].
//! That trait describes how to render the front and back of the card and how cards
//! compare or match with one another.
//!
//! # Examples
//!
//! ```
//! use gametools::{Card, CardFaces};
//!
//! #[derive(Clone)]
//! struct EmojiCard(&'static str);
//!
//! impl CardFaces for EmojiCard {
//!     fn display_front(&self) -> String { self.0.to_string() }
//!     fn display_back(&self) -> Option<String> { Some(String::from("🎴")) }
//!     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
//!     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(other.0) }
//! }
//!
//! let card = Card::new_card(EmojiCard("🂡"));
//! assert!(card.face_up);
//! assert!(card.deck_id.is_none());
//! ```

use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Deck;

use super::deck::DeckId;

/// Describes how to work with the front and back of a card, as well as how to compare
/// or match two cards of the same type.
///
/// Importantly, here "matches" means that the cards are completely interchangeable for
/// ownership / removal / collection management purposes. Specific games will need to be
/// able to match only on certain combinatons of aspects of their cards' faces, which
/// should be implemented as methods on the specific card type.
pub trait CardFaces {
    fn display_front(&self) -> String;
    fn display_back(&self) -> Option<String>;
    fn matches(&self, other: &Self) -> bool;
    fn compare(&self, other: &Self) -> std::cmp::Ordering;
}

/// A generic card of any kind, as long as it has faces.
///
/// The [`faces`](Self::faces) field stores application-specific information while the
/// remaining fields are convenience metadata maintained by the library.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Card<T: CardFaces> {
    /// The user-defined information that appears on the card.
    pub faces: T,
    /// A globally unique identifier assigned on creation.
    pub uuid: Uuid,
    /// The deck that currently owns this card, if any.
    pub deck_id: Option<DeckId>,
    /// Whether the front face of the card is currently visible.
    pub face_up: bool,
}

impl<T: CardFaces> From<T> for Card<T> {
    fn from(faces: T) -> Self {
        Card {
            faces,
            uuid: Uuid::new_v4(),
            deck_id: None,
            face_up: true,
        }
    }
}

impl<T: CardFaces> Card<T> {
    /// Create a new card from a struct that is CardFaces.
    ///
    /// By default, there are orphan or dummy cards that don't belong to a Deck. A DeckId
    /// is assigned to them if they are passed through Deck::new() or Deck::new_from_faces().
    ///
    /// ```
    /// use gametools::{Card, CardFaces};
    ///
    /// #[derive(Clone)]
    /// struct Number(u8);
    ///
    /// impl CardFaces for Number {
    ///     fn display_front(&self) -> String { format!("N{}", self.0) }
    ///     fn display_back(&self) -> Option<String> { None }
    ///     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
    ///     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
    /// }
    ///
    /// let card = Card::new_card(Number(7));
    /// assert!(card.face_up);
    /// assert!(card.deck_id.is_none());
    /// ```
    pub fn new_card(faces: T) -> Card<T> {
        Card {
            faces,
            uuid: Uuid::new_v4(),
            deck_id: None,
            face_up: true,
        }
    }
    /// Flip the card over.
    ///
    /// This changes which side of the card is visible. Display is implemented so that printing
    /// {card} shows the face from whichever side is up.
    ///
    /// ```
    /// use gametools::{Card, CardFaces};
    ///
    /// #[derive(Clone)]
    /// struct Stub;
    ///
    /// impl CardFaces for Stub {
    ///     fn display_front(&self) -> String { String::from("front") }
    ///     fn display_back(&self) -> Option<String> { Some(String::from("back")) }
    ///     fn matches(&self, _other: &Self) -> bool { true }
    ///     fn compare(&self, _other: &Self) -> std::cmp::Ordering { std::cmp::Ordering::Equal }
    /// }
    ///
    /// let mut card = Card::new_card(Stub);
    /// assert!(card.face_up);
    /// card.flip();
    /// assert!(!card.face_up);
    /// ```
    pub fn flip(&mut self) {
        self.face_up = !self.face_up;
    }
    /// Determine whether this card belongs to a specific `Deck`.
    ///
    /// ```
    /// use gametools::{Card, CardFaces, Deck};
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
    /// let mut deck = Deck::from_cards("demo", [Card::new_card(Face(1))]);
    /// let card = deck.cards()[0].clone();
    /// let deck_copy = deck.clone();
    /// assert!(card.is_from_deck(&deck_copy));
    /// ```
    pub fn is_from_deck(&self, deck: &Deck<T>) -> bool {
        self.deck_id == Some(deck.deck_id())
    }

    /// Force a card to belong to the deck identified by `deck_id`.
    ///
    /// This is normally handled automatically by [`Deck::new`](crate::cards::deck::Deck::new),
    /// but individual games may need to reassign cards in custom flows.
    pub fn assign_to_deck(&mut self, deck_id: DeckId) {
        self.deck_id = Some(deck_id);
    }
}

impl<T: CardFaces> std::fmt::Display for Card<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.face_up {
            write!(f, "{}", self.faces.display_front())
        } else {
            write!(
                f,
                "{}",
                self.faces
                    .display_back()
                    .unwrap_or_else(|| "|Face Down|".to_string())
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct StubFaces {
        front: &'static str,
        back: Option<&'static str>,
        match_id: u8,
        score: i16,
    }

    impl CardFaces for StubFaces {
        fn display_front(&self) -> String {
            self.front.to_string()
        }

        fn display_back(&self) -> Option<String> {
            self.back.map(|s| s.to_string())
        }

        fn matches(&self, other: &Self) -> bool {
            self.match_id == other.match_id
        }

        fn compare(&self, other: &Self) -> std::cmp::Ordering {
            self.score.cmp(&other.score)
        }
    }

    fn make_card(score: i16, back: Option<&'static str>) -> Card<StubFaces> {
        Card::new_card(StubFaces {
            front: "front",
            back,
            match_id: 7,
            score,
        })
    }

    #[test]
    fn new_card_initializes_face_up_with_unique_faces() {
        let card = make_card(3, Some("back"));

        assert!(card.face_up);
        assert_eq!(card.faces.display_front(), "front");
    }

    #[test]
    fn flip_toggles_face_orientation() {
        let mut card = make_card(0, None);

        card.flip();
        assert!(!card.face_up);

        card.flip();
        assert!(card.face_up);
    }

    #[test]
    fn display_shows_front_when_face_up() {
        let card = make_card(1, Some("back"));
        assert_eq!(card.to_string(), "front");
    }

    #[test]
    fn display_prefers_back_when_face_down_and_available() {
        let mut card = make_card(1, Some("back"));
        card.flip();

        assert_eq!(card.to_string(), "back");
    }

    #[test]
    fn display_uses_default_when_face_down_without_back() {
        let mut card = make_card(2, None);
        card.flip();

        assert_eq!(card.to_string(), "|Face Down|");
    }

    #[test]
    fn compare_returns_expected_outcomes() {
        let low = StubFaces {
            front: "front",
            back: None,
            match_id: 1,
            score: 1,
        };
        let mid = StubFaces {
            score: 2,
            ..low.clone()
        };
        let high = StubFaces {
            score: 3,
            ..low.clone()
        };

        assert_eq!(mid.compare(&low), std::cmp::Ordering::Greater);
        assert_eq!(mid.compare(&high), std::cmp::Ordering::Less);
        assert_eq!(mid.compare(&mid), std::cmp::Ordering::Equal);
    }
}
