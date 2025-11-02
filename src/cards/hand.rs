//! hand module
//!
//! A `Hand` of `Card<T>`
//!
use crate::cards::{AddCard, Card, CardCollection, CardFaces, TakeCard};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Hand<T: CardFaces> {
    pub player: String,
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
    pub fn new(player: &str) -> Self {
        Self {
            player: player.to_string(),
            cards: Vec::<Card<T>>::new(),
        }
    }
}

impl<T: CardFaces> AddCard<T> for Hand<T> {
    fn add_card(&mut self, card: Card<T>) {
        self.cards.push(card);
    }
}

impl<T: CardFaces> TakeCard<T> for Hand<T> {
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
