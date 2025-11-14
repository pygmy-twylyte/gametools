//! # Decks
//!
//! A [`Deck`] starts full and only loses cards through draw-like operations. It is the
//! canonical source of cards for most games, pairing neatly with [`Hand`]
//! or [`Pile`](crate::cards::pile::Pile) to model game flow.
//!
//! # Examples
//!
//! ```
//! use gametools::{Card, CardFaces, CardCollection, Deck, TakeCard};
//!
//! #[derive(Clone)]
//! struct Number(u8);
//!
//! impl CardFaces for Number {
//!     fn display_front(&self) -> String { format!("N{}", self.0) }
//!     fn display_back(&self) -> Option<String> { None }
//!     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
//!     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
//! }
//!
//! let cards = (1..=3).map(|n| Card::new_card(Number(n))).collect::<Vec<_>>();
//! let mut deck = Deck::from_cards("numbers", cards);
//! assert_eq!(deck.size(), 3);
//! let drawn = deck.take_card().unwrap();
//! assert_eq!(drawn.faces.0, 3);
//! assert_eq!(deck.size(), 2);
//! ```

use crate::cards::{AddCard, Card, CardCollection, CardFaces, Hand, TakeCard};
use rand::prelude::SliceRandom;
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Uniquely identifies a specific deck instance.
pub struct DeckId(Uuid);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A named collection of cards that are dealt from top to bottom.
pub struct Deck<T: CardFaces> {
    /// Friendly name used to describe the deck.
    pub name: String,
    /// Unique identifier automatically assigned at deck creation.
    deck_id: DeckId,
    /// Cards stored with the "top" card at the end of the vector.
    cards: Vec<Card<T>>,
}
impl<T: CardFaces + Clone> Deck<T> {
    /// Create a new, empty `Deck`
    pub fn new() -> Self {
        Self {
            name: String::new(),
            deck_id: DeckId(Uuid::new_v4()),
            cards: Vec::new(),
        }
    }
    /// Build a deck from an owned vector of [`Card`]s.
    ///
    /// Each card is tagged with the deck's [`DeckId`], allowing downstream code
    /// to check ownership.
    ///
    /// ```
    /// use gametools::{Card, CardCollection, CardFaces, Deck};
    ///
    /// #[derive(Clone)]
    /// struct Face(u8);
    ///
    /// impl CardFaces for Face {
    ///     fn display_front(&self) -> String { format!("Card {}", self.0) }
    ///     fn display_back(&self) -> Option<String> { None }
    ///     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
    ///     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
    /// }
    ///
    /// let cards = vec![Card::new_card(Face(1)), Card::new_card(Face(2))];
    /// let deck = Deck::from_cards("demo", cards);
    /// assert!(deck
    ///     .cards()
    ///     .iter()
    ///     .all(|card| card.deck_id == Some(deck.deck_id())));
    /// ```
    pub fn from_cards(name: &str, cards: impl IntoIterator<Item = Card<T>>) -> Self {
        let deck_id = DeckId(Uuid::new_v4());
        Self {
            name: name.to_string(),
            deck_id,
            cards: cards
                .into_iter()
                .map(|mut c| {
                    c.assign_to_deck(deck_id);
                    c
                })
                .collect::<Vec<_>>(),
        }
    }

    /// Create a deck by supplying raw face values that will be wrapped in [`Card`]s.
    ///
    /// The faces are consumed by this constructor. If you want to retain faces to build
    /// additional [`Deck`]s, use [`Deck::with_borrowed_faces`] instead.
    pub fn from_faces(name: &str, faces: impl IntoIterator<Item = T>) -> Self {
        let deck_id = DeckId(Uuid::new_v4());
        Self {
            name: name.to_string(),
            deck_id,
            cards: faces
                .into_iter()
                .map(|face| {
                    let mut card = Card::from(face);
                    card.assign_to_deck(deck_id);
                    card
                })
                .collect(),
        }
    }
    /// Create a deck by supplying raw face values that will be wrapped in [`Card`]s.
    ///
    /// The faces are not consumed, but each face is cloned in order to create the corresponding [`Card`]s.
    /// ```
    /// use gametools::{CardFaces, Deck};
    ///
    /// #[derive(Clone)]
    /// struct Face(&'static str);
    ///
    /// impl CardFaces for Face {
    ///     fn display_front(&self) -> String { self.0.to_string() }
    ///     fn display_back(&self) -> Option<String> { None }
    ///     fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
    ///     fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
    /// }
    ///
    /// let faces = vec![Face("A"), Face("B")];
    /// let deck = Deck::with_borrowed_faces("demo", &faces);
    /// assert_eq!(deck.cards().len(), 2);
    /// ```
    pub fn with_borrowed_faces(name: &str, faces: &[T]) -> Self {
        let cards = faces
            .iter()
            .map(|f| Card::from(f.clone()))
            .collect::<Vec<_>>();
        Self::from_cards(name, cards)
    }
}

impl<T: CardFaces> Deck<T> {
    /// Obtain a slice of the cards remaining in the deck.
    pub fn cards(&self) -> &[Card<T>] {
        &self.cards
    }

    /// Get the unique identifier for this deck.
    pub fn deck_id(&self) -> DeckId {
        self.deck_id
    }

    /// Randomly permute the order of cards in the deck.
    ///
    /// ```
    /// use gametools::{Card, CardCollection, CardFaces, Deck};
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
    /// let cards = (0..5).map(|n| Card::new_card(Face(n))).collect::<Vec<_>>();
    /// let mut deck = Deck::from_cards("demo", cards);
    /// deck.shuffle(); // order changes, but size remains the same
    /// assert_eq!(deck.cards().len(), 5);
    /// ```
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::rng());
    }

    /// Determine whether the supplied `Card` belongs to this `Deck`.
    ///
    /// ```
    /// use gametools::{Card, CardCollection, CardFaces, Deck};
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
    /// let deck = Deck::from_cards("demo", [Card::new_card(Face(1))]);
    /// let card = deck.cards()[0].clone();
    /// assert!(deck.owns_card(&card));
    ///
    /// let other_deck = Deck::from_cards("other", [Card::new_card(Face(2))]);
    /// assert!(!other_deck.owns_card(&card));
    /// ```
    pub fn owns_card(&self, card: &Card<T>) -> bool {
        if let Some(card_deck_id) = &card.deck_id {
            self.deck_id == *card_deck_id
        } else {
            false
        }
    }

    /// Deal `count` cards to each player in `players`, returning the resulting hands.
    ///
    /// Cards are dealt round-robin: the first player receives the first card, the
    /// second player receives the next, and so on.
    ///
    /// ```
    /// use gametools::{Card, CardCollection, CardFaces, Deck};
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
    /// let cards = (1..=4).map(|n| Card::new_card(Face(n))).collect::<Vec<_>>();
    /// let mut deck = Deck::from_cards("demo", cards);
    /// let hands = deck.deal(&["alice", "bob"], 1);
    ///
    /// assert_eq!(hands[0].player, "alice");
    /// assert_eq!(hands[0].cards.len(), 1);
    /// assert_eq!(hands[1].player, "bob");
    /// assert_eq!(deck.size(), 2);
    /// ```
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

impl<T: CardFaces> Default for Deck<T> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            deck_id: DeckId(Uuid::new_v4()),
            cards: Default::default(),
        }
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
    /// Draw the next card from the deck. Returns `None` when empty.
    fn take_card(&mut self) -> Option<Card<T>> {
        self.cards.pop()
    }

    /// Remove the first card whose faces match the supplied `search_card`.
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

        let deck = Deck::with_borrowed_faces("test", &faces);

        assert_eq!(deck.name, "test");
        let ids: Vec<u8> = deck.cards.iter().map(|c| c.faces.id).collect();
        assert_eq!(ids, vec![1, 2]);
        // ensure original faces untouched
        assert_eq!(faces[0].id, 1);
        assert!(
            deck.cards
                .iter()
                .all(|card| card.deck_id == Some(deck.deck_id))
        );
    }

    #[test]
    fn new_assigns_deck_id_to_all_cards() {
        let deck = Deck::from_cards("test", [make_card(1), make_card(2)]);

        assert!(
            deck.cards
                .iter()
                .all(|card| card.deck_id == Some(deck.deck_id))
        );
        assert!(deck.cards.iter().all(|card| card.deck_id.is_some()));
    }

    #[test]
    fn take_card_removes_last_card() {
        let mut deck = Deck::from_cards("test", [make_card(1), make_card(2)]);

        let taken = deck.take_card().unwrap();

        assert_eq!(taken.faces.id, 2);
        assert_eq!(deck.cards.len(), 1);
    }

    #[test]
    fn take_match_removes_matching_card() {
        let mut deck = Deck::from_cards("test", [make_card(1), make_card(2), make_card(3)]);
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
        let mut deck = Deck::from_cards("test", cards);
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
        let mut deck = Deck::from_cards("test", cards);
        let players = vec!["a", "b"];

        let hands = deck.deal(&players, 2);

        let lengths: Vec<usize> = hands.iter().map(|hand| hand.cards.len()).collect();
        assert_eq!(lengths, vec![2, 1]);
        assert!(deck.cards.is_empty());
    }

    #[test]
    fn owns_card_identifies_membership() {
        let deck = Deck::from_cards("test", [make_card(1), make_card(2)]);
        let deck_card = deck.cards[0].clone();
        let mut other_deck = Deck::from_cards("other", [make_card(3)]);
        let other_card = other_deck.take_card().expect("card expected");

        assert!(deck.owns_card(&deck_card));
        assert!(!deck.owns_card(&other_card));
    }
}
