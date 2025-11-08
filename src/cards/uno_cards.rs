//! Uno Card Module

use crate::{cards::CardFaces, Card};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnoCard {
    pub color: UnoColor,
    pub kind: UnoCardKind,
}

impl CardFaces for UnoCard {
    fn display_front(&self) -> String {
        format!("{} ({})", self.color, self.kind)
    }

    fn display_back(&self) -> Option<String> {
        None
    }

    fn matches(&self, other: &Self) -> bool {
        self.color == other.color && self.kind == other.kind
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        if self.kind.is_wild() {
            std::cmp::Ordering::Greater
        } else if other.kind.is_wild() {
            std::cmp::Ordering::Less
        } else {
            self.kind.cmp(&other.kind)
        }
        .then_with(|| self.color.cmp(&other.color))
    }
}

impl UnoCard {
    /// Returns whether this card can be legally played on another card.
    pub fn plays_on(&self, other: &UnoCard, declared_color: Option<UnoColor>) -> bool {
        use UnoCardKind::*;
        if let Some(declared) = declared_color
        && self.color == declared {
                return true;
            }

        if self.color == other.color {
            return true;
        }
        match self.kind {
            Wild | WildDrawFour => true,
            Number(x) => {
                if let Number(other) = other.kind {
                    x == other
                } else {
                    false
                }
            }
            Action(uno_action) => match uno_action {
                DrawTwo => matches!(other.kind, DrawTwo),
                Skip => matches!(other.kind, Skip),
                Reverse => matches!(other.kind, Reverse),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnoColor {
    Red,
    Blue,
    Green,
    Yellow,
    Black,
}
impl std::fmt::Display for UnoColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnoColor::Red => write!(f, "Red"),
            UnoColor::Blue => write!(f, "Blue"),
            UnoColor::Green => write!(f, "Green"),
            UnoColor::Yellow => write!(f, "Yellow"),
            UnoColor::Black => write!(f, "Black"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnoCardKind {
    Number(u8),
    Action(UnoAction),
    Wild,
    WildDrawFour,
}
impl UnoCardKind {
    pub fn is_wild(&self) -> bool {
        matches!(self, Self::Wild | Self::WildDrawFour)
    }
}
impl std::fmt::Display for UnoCardKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnoCardKind::Number(number) => write!(f, "#{}", number),
            UnoCardKind::Action(action) => write!(f, "{}!", action),
            UnoCardKind::Wild => write!(f, "Wild"),
            UnoCardKind::WildDrawFour => write!(f, "Wild + Draw 4"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnoAction {
    DrawTwo,
    Skip,
    Reverse,
}
impl std::fmt::Display for UnoAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnoAction::DrawTwo => write!(f, "Draw Two"),
            UnoAction::Skip => write!(f, "Skip"),
            UnoAction::Reverse => write!(f, "Reverse"),
        }
    }
}

/// Counts of each number card per color, 0-9.
pub const UNO_NUMBER_CARD_COUNTS: &[u8] = &[1, 2, 2, 2, 2, 2, 2, 2, 2, 2];
pub const MAIN_UNO_COLORS: &[UnoColor] = &[
    UnoColor::Red,
    UnoColor::Blue,
    UnoColor::Green,
    UnoColor::Yellow,
];

/// Create a full set of 108 Uno cards
pub fn full_uno_set() -> Vec<UnoCard> {
    let mut cards = Vec::new();
    cards.extend(uno_number_cards());
    cards.extend(uno_action_cards());
    cards.extend(uno_wild_cards());
    cards
}

/// Create all of the number card faces for a standard Uno deck
pub fn uno_number_cards() -> Vec<UnoCard> {
    let mut cards = Vec::new();
    for color in MAIN_UNO_COLORS {
        for (number, count) in UNO_NUMBER_CARD_COUNTS.iter().enumerate() {
            for _ in 0..*count {
                cards.push(UnoCard {
                    color: *color,
                    kind: UnoCardKind::Number(number as u8),
                })
            }
        }
    }
    cards
}

/// Create all of the action cards in a standard Uno deck
pub fn uno_action_cards() -> Vec<UnoCard> {
    let mut cards = Vec::new();
    for color in MAIN_UNO_COLORS {
        for _ in 0..2 {
            cards.push(UnoCard {
                color: *color,
                kind: UnoCardKind::Action(UnoAction::DrawTwo),
            });
            cards.push(UnoCard {
                color: *color,
                kind: UnoCardKind::Action(UnoAction::Skip),
            });
            cards.push(UnoCard {
                color: *color,
                kind: UnoCardKind::Action(UnoAction::Reverse),
            });
        }
    }
    cards
}

/// Create the wild cards for a standard Uno deck
pub fn uno_wild_cards() -> Vec<UnoCard> {
    let mut cards = Vec::new();
    for _ in 0..4 {
        cards.push(UnoCard {
            color: UnoColor::Black,
            kind: UnoCardKind::Wild,
        });
        cards.push(UnoCard {
            color: UnoColor::Black,
            kind: UnoCardKind::WildDrawFour,
        });
    }
    cards
}

impl super::Hand<UnoCard> {
    pub fn playable_on(
        &self,
        top: &Card<UnoCard>,
        declared_color: Option<UnoColor>,
    ) -> Vec<(usize, &Card<UnoCard>)> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn number_cards_follow_expected_distribution() {
        let cards = uno_number_cards();
        let expected_per_color: usize = UNO_NUMBER_CARD_COUNTS
            .iter()
            .map(|&count| count as usize)
            .sum();

        assert_eq!(
            cards.len(),
            MAIN_UNO_COLORS.len() * expected_per_color,
            "Unexpected number of Uno number cards"
        );
        assert!(cards
            .iter()
            .all(|card| matches!(card.kind, UnoCardKind::Number(_))));

        for color in MAIN_UNO_COLORS {
            for number in 0u8..=9 {
                let expected = UNO_NUMBER_CARD_COUNTS[number as usize] as usize;
                let actual = cards
                    .iter()
                    .filter(|card| {
                        card.color == *color
                            && matches!(card.kind, UnoCardKind::Number(value) if value == number)
                    })
                    .count();
                assert_eq!(
                    actual, expected,
                    "Expected {expected} copies of {color:?} {number}, found {actual}"
                );
            }
        }
    }

    #[test]
    fn action_cards_include_two_of_each_per_color() {
        let cards = uno_action_cards();
        assert_eq!(
            cards.len(),
            MAIN_UNO_COLORS.len() * 6,
            "Unexpected number of Uno action cards"
        );

        for color in MAIN_UNO_COLORS {
            for action in [UnoAction::DrawTwo, UnoAction::Skip, UnoAction::Reverse] {
                let actual = cards
                    .iter()
                    .filter(|card| {
                        card.color == *color
                            && matches!(card.kind, UnoCardKind::Action(kind) if kind == action)
                    })
                    .count();
                assert_eq!(
                    actual, 2,
                    "Expected two {color:?} {action:?} cards, found {actual}"
                );
            }
        }
    }

    #[test]
    fn wild_cards_include_four_of_each_type() {
        let cards = uno_wild_cards();
        assert_eq!(cards.len(), 8);
        assert!(cards.iter().all(|card| card.color == UnoColor::Black));

        let wild_count = cards
            .iter()
            .filter(|card| matches!(card.kind, UnoCardKind::Wild))
            .count();
        let wild_draw_four_count = cards
            .iter()
            .filter(|card| matches!(card.kind, UnoCardKind::WildDrawFour))
            .count();

        assert_eq!(wild_count, 4);
        assert_eq!(wild_draw_four_count, 4);
    }

    #[test]
    fn full_uno_set_contains_expected_cards() {
        let full_set = full_uno_set();
        assert_eq!(full_set.len(), 108);

        let mut seen = BTreeMap::new();
        for card in &full_set {
            *seen.entry(*card).or_insert(0usize) += 1;
        }

        for card in uno_number_cards()
            .into_iter()
            .chain(uno_action_cards())
            .chain(uno_wild_cards())
        {
            let should_remove = match seen.get_mut(&card) {
                Some(count) if *count > 0 => {
                    *count -= 1;
                    *count == 0
                }
                _ => panic!("Card {:?} was expected in full Uno set but not found", card),
            };

            if should_remove {
                seen.remove(&card);
            }
        }

        assert!(
            seen.is_empty(),
            "Found unexpected extra cards in full Uno set: {:?}",
            seen
        );
    }
}
