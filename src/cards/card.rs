//!
//! card module
//!
//! Implements a card generic over T, where T: `CardFaces`. The `CardFaces` trait
//! defines how each side of the card appears and how they compare to each other.

use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A generic card of any kind, as long as it has faces.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Card<T: CardFaces> {
    pub faces: T,
    pub uuid: Uuid,
    pub face_up: bool,
}

pub trait CardFaces {
    fn display_front(&self) -> String;
    fn display_back(&self) -> Option<String>;
    fn matches(&self, other: &Self) -> bool;
    fn compare(&self, other: &Self) -> std::cmp::Ordering;
}

impl<T: CardFaces> Card<T> {
    pub fn new_card(faces: T) -> Card<T> {
        Card {
            faces,
            uuid: Uuid::new_v4(),
            face_up: true,
        }
    }
    pub fn flip(&mut self) {
        self.face_up = !self.face_up;
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
