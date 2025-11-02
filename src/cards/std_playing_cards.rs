//! # Standard Playing Card
//!
//! This defines a standard playing card, with ranks and suits, for use with the cards module.
//! Also included are method implementations for Hand and Deck specific to standard rank/suit cards.
use crate::cards::{Card, CardFaces, Hand};
use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A standard playing card that you'd find in a typical deck of 52 (54 with Jokers).
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StandardCard {
    pub suit: Suit,
    pub rank: Rank,
    pub value: u8,
}
impl StandardCard {
    pub fn new_card(rank: Rank, suit: Suit) -> Self {
        Self {
            rank,
            suit,
            value: rank as u8,
        }
    }
}
impl CardFaces for StandardCard {
    fn display_front(&self) -> String {
        format!("{}.{}", self.rank, self.suit)
    }

    fn display_back(&self) -> Option<String> {
        None
    }

    fn matches(&self, other: &Self) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

/// Ranks for "normal" cards (Jokers treated separately).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Joker = 255,
}
impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Rank::Two => write!(f, "2"),
            Rank::Three => write!(f, "3"),
            Rank::Four => write!(f, "4"),
            Rank::Five => write!(f, "5"),
            Rank::Six => write!(f, "6"),
            Rank::Seven => write!(f, "7"),
            Rank::Eight => write!(f, "8"),
            Rank::Nine => write!(f, "9"),
            Rank::Ten => write!(f, "10"),
            Rank::Jack => write!(f, "J"),
            Rank::Queen => write!(f, "Q"),
            Rank::King => write!(f, "K"),
            Rank::Ace => write!(f, "A"),
            Rank::Joker => write!(f, "*"),
        }
    }
}
impl Rank {
    /// Returns a list of the standard ranks (no Joker)
    pub fn normal_ranks() -> Vec<Rank> {
        vec![
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ]
    }
    /// Returns a list of all ranks (Joker included)
    pub fn all_ranks() -> Vec<Rank> {
        let mut all_ranks = Rank::normal_ranks();
        all_ranks.push(Rank::Joker);
        all_ranks
    }

    pub fn from_value(value: u8) -> Option<Rank> {
        match value {
            1 | 14 => Some(Rank::Ace),
            2 => Some(Rank::Two),
            3 => Some(Rank::Three),
            4 => Some(Rank::Four),
            5 => Some(Rank::Five),
            6 => Some(Rank::Six),
            7 => Some(Rank::Seven),
            8 => Some(Rank::Eight),
            9 => Some(Rank::Nine),
            10 => Some(Rank::Ten),
            11 => Some(Rank::Jack),
            12 => Some(Rank::Queen),
            13 => Some(Rank::King),
            _ => None,
        }
    }
}

/// Suits for "normal" playing cards
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Suit {
    Clubs,
    Hearts,
    Diamonds,
    Spades,
    Wild,
}
impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Suit::Clubs => write!(f, "♣"),
            Suit::Hearts => write!(f, "♥"),
            Suit::Diamonds => write!(f, "♦"),
            Suit::Spades => write!(f, "♠"),
            Suit::Wild => write!(f, "?"),
        }
    }
}
impl Suit {
    /// Returns a list of normal suits (Wild excluded).
    pub fn normal_suits() -> Vec<Suit> {
        vec![Suit::Clubs, Suit::Hearts, Suit::Diamonds, Suit::Spades]
    }
    /// Returns a list of all suits (including Wild).
    pub fn all_suits() -> Vec<Suit> {
        let mut all_suits = Suit::normal_suits();
        all_suits.push(Suit::Wild);
        all_suits
    }
}

/// Create all 52 cards for a stanard deck.
pub fn full_deck() -> Vec<StandardCard> {
    let mut deck = Vec::new();
    for suit in Suit::normal_suits() {
        for rank in Rank::normal_ranks() {
            deck.push(StandardCard::new_card(rank, suit));
        }
    }
    deck
}

/// Create all 52 cards for a standard deck, plus two Jokers.
pub fn full_deck_with_jokers() -> Vec<StandardCard> {
    let mut deck = full_deck();
    deck.push(StandardCard::new_card(Rank::Joker, Suit::Wild));
    deck.push(StandardCard::new_card(Rank::Joker, Suit::Wild));
    deck
}

impl Hand<StandardCard> {
    /// Check whether a card matching a rank and suit is in the `Hand`.
    pub fn contains(&self, rank: Rank, suit: Suit) -> bool {
        let search = StandardCard::new_card(rank, suit);
        self.cards.iter().any(|card| card.faces.matches(&search))
    }

    /// Count how many cards in the hand have a specific rank.
    pub fn count_rank(&self, rank: Rank) -> usize {
        self.cards.iter().filter(|c| c.faces.rank == rank).count()
    }

    /// Create a map of `Rank` counts for the current `Hand`.
    pub fn rank_map(&self) -> BTreeMap<Rank, usize> {
        let mut rank_map = BTreeMap::new();
        for card in &self.cards {
            rank_map
                .entry(card.faces.rank)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        rank_map
    }

    /// Count how many cards in the hand have a specific suit.
    pub fn count_suit(&self, suit: Suit) -> usize {
        self.cards.iter().filter(|c| c.faces.suit == suit).count()
    }

    /// Create a map of `Suit` counts for the current `Hand`.
    pub fn suit_map(&self) -> BTreeMap<Suit, usize> {
        let mut suit_map = BTreeMap::new();
        for card in &self.cards {
            suit_map
                .entry(card.faces.suit)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        suit_map
    }

    /// Returns true if every card in the hand belongs to the same `Suit`.
    pub fn is_flush(&self) -> bool {
        let total_cards = self.cards.len();
        if total_cards == 0 {
            return false;
        }

        let mut suit_map = self.suit_map();
        let wildcards = suit_map.remove(&Suit::Wild).unwrap_or(0);

        if suit_map.is_empty() {
            return wildcards >= total_cards;
        }

        suit_map
            .values()
            .any(|count| *count + wildcards >= total_cards)
    }

    /// Check for N cards of a kind.
    ///
    /// Returns `None` if none reach N, or `Some` cards if there are any.
    pub fn find_n_of_a_kind(&self, count: usize) -> Option<Vec<&StandardCard>> {
        if count == 0 {
            return Some(Vec::new());
        }
        if self.cards.len() < count {
            return None;
        }

        let mut rank_groups: BTreeMap<Rank, Vec<&Card<StandardCard>>> = BTreeMap::new();
        for card in &self.cards {
            rank_groups.entry(card.faces.rank).or_default().push(card);
        }

        let jokers = rank_groups.remove(&Rank::Joker).unwrap_or_default();
        let mut groups: Vec<(Rank, Vec<&Card<StandardCard>>)> = rank_groups.into_iter().collect();
        groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

        for (_rank, cards) in groups {
            let mut result: Vec<&StandardCard> =
                cards.iter().take(count).map(|card| &card.faces).collect();
            if result.len() == count {
                return Some(result);
            }

            let missing = count - result.len();
            if !jokers.is_empty() && jokers.len() >= missing {
                result.extend(jokers.iter().take(missing).map(|card| &card.faces));
                return Some(result);
            }
        }

        if jokers.len() >= count {
            return Some(jokers.iter().take(count).map(|card| &card.faces).collect());
        }

        None
    }

    /// Returns `Some` ordered cards that form a straight of the requested length, or `None` if no straight exists.
    ///
    /// Jokers serve as wild cards and may fill any missing rank. Aces may be used high or low.
    pub fn find_n_straight(&self, count: usize) -> Option<Vec<&StandardCard>> {
        // handle edge cases with obvious results
        if count == 0 {
            return Some(Vec::new());
        }
        if self.cards.len() < count || count > 14 {
            return None;
        }

        // group cards according their ranks
        let mut rank_groups: BTreeMap<Rank, Vec<&Card<StandardCard>>> = BTreeMap::new();
        for card in &self.cards {
            rank_groups.entry(card.faces.rank).or_default().push(card);
        }

        // pull the wildcards out of the rank groups, counting them for later insertion if
        // needed to complete a straight
        let jokers = rank_groups.remove(&Rank::Joker).unwrap_or_default();

        // if all of the cards were wild, we have a straight if we have enough cards
        if rank_groups.is_empty() {
            return (jokers.len() >= count)
                .then(|| jokers.iter().take(count).map(|card| &card.faces).collect());
        }

        // any sequence starting after this will run out of ranks before we have enough
        let max_start = 14usize.saturating_sub(count).saturating_add(1);

        // try to pull a card from `count` consecutive rank groups, inserting Jokers
        // as needed and if available
        for start in 1..=max_start {
            let mut available = rank_groups.clone();
            let mut jokers_left = jokers.clone();
            let mut straight_cards: Vec<&Card<StandardCard>> = Vec::with_capacity(count);
            let mut success = true;

            for offset in 0..count {
                let value = (start + offset) as u8;
                let Some(rank) = Rank::from_value(value) else {
                    success = false;
                    break;
                };

                // if there's a natural card to fill this rank slot, use it and move on
                if let Some(cards) = available.get_mut(&rank) {
                    if let Some(card) = cards.pop() {
                        straight_cards.push(card);
                        continue;
                    }
                }

                // if there's Joker to fill this rank slot, use it and move on
                if let Some(joker_card) = jokers_left.pop() {
                    straight_cards.push(joker_card);
                } else {
                    success = false;
                    break;
                }
            }

            if success {
                let straight_faces = straight_cards.iter().map(|card| &card.faces).collect();
                return Some(straight_faces);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Card;
    use std::collections::BTreeSet;

    #[test]
    fn standard_card_constructor_sets_rank_suit_and_value() {
        let card = StandardCard::new_card(Rank::Ace, Suit::Spades);

        assert_eq!(card.rank, Rank::Ace);
        assert_eq!(card.suit, Suit::Spades);
        assert_eq!(card.value, Rank::Ace as u8);
    }

    #[test]
    fn standard_card_display_and_faces_behave_as_expected() {
        let card = StandardCard::new_card(Rank::Ten, Suit::Hearts);

        assert_eq!(card.display_front(), "10.♥");
        assert!(card.display_back().is_none());
    }

    #[test]
    fn matches_and_compare_follow_rank_and_suit() {
        let low = StandardCard::new_card(Rank::Five, Suit::Clubs);
        let high = StandardCard::new_card(Rank::Seven, Suit::Clubs);
        let different_suit = StandardCard::new_card(Rank::Five, Suit::Hearts);

        assert!(low.matches(&StandardCard::new_card(Rank::Five, Suit::Clubs)));
        assert!(!low.matches(&different_suit));
        assert_eq!(low.compare(&high), std::cmp::Ordering::Less);
        assert_eq!(high.compare(&low), std::cmp::Ordering::Greater);
        assert_eq!(low.compare(&low), std::cmp::Ordering::Equal);
    }

    #[test]
    fn normal_ranks_and_suits_return_expected_sets() {
        let ranks = Rank::normal_ranks();
        let suits = Suit::normal_suits();

        assert_eq!(ranks.len(), 13);
        assert!(ranks.iter().all(|rank| *rank != Rank::Joker));
        assert_eq!(suits.len(), 4);
        assert!(!suits.contains(&Suit::Wild));
    }

    #[test]
    fn full_deck_contains_all_rank_suit_combinations() {
        let deck = full_deck();

        assert_eq!(deck.len(), 52);
        let unique: BTreeSet<(Rank, Suit)> = deck.iter().map(|c| (c.rank, c.suit)).collect();
        assert_eq!(unique.len(), 52);
    }

    #[test]
    fn full_deck_with_jokers_adds_two_wild_cards() {
        let deck = full_deck_with_jokers();

        assert_eq!(deck.len(), 54);
        let joker_count = deck
            .iter()
            .filter(|c| c.rank == Rank::Joker && c.suit == Suit::Wild)
            .count();
        assert_eq!(joker_count, 2);
    }

    #[test]
    fn hand_detects_ace_low_straight() {
        let mut hand = Hand::new("player");
        let cards = [
            (Rank::Ace, Suit::Spades),
            (Rank::Two, Suit::Clubs),
            (Rank::Three, Suit::Diamonds),
            (Rank::Four, Suit::Hearts),
            (Rank::Five, Suit::Spades),
        ];

        for (rank, suit) in cards {
            hand.cards
                .push(Card::new_card(StandardCard::new_card(rank, suit)));
        }

        let straight = hand.find_n_straight(5).expect("expected ace-low straight");
        let ranks: Vec<Rank> = straight.iter().map(|card| card.rank).collect();
        assert_eq!(
            ranks,
            vec![Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five]
        );
    }

    #[test]
    fn hand_detects_straight_with_joker() {
        let mut hand = Hand::new("player");
        let cards = [
            (Rank::Ten, Suit::Hearts),
            (Rank::Queen, Suit::Diamonds),
            (Rank::King, Suit::Clubs),
            (Rank::Ace, Suit::Spades),
        ];

        for (rank, suit) in cards {
            hand.cards
                .push(Card::new_card(StandardCard::new_card(rank, suit)));
        }
        hand.cards.push(Card::new_card(StandardCard::new_card(
            Rank::Joker,
            Suit::Wild,
        )));

        let straight = hand
            .find_n_straight(5)
            .expect("expected straight with joker");
        assert_eq!(straight.len(), 5);
        assert_eq!(
            straight
                .iter()
                .filter(|card| card.rank == Rank::Joker)
                .count(),
            1
        );
        let mut ranks: Vec<_> = straight.iter().map(|card| card.rank).collect();
        ranks.sort();
        assert!(ranks.contains(&Rank::Ten));
        assert!(ranks.contains(&Rank::Queen));
        assert!(ranks.contains(&Rank::King));
        assert!(ranks.contains(&Rank::Ace));
    }

    #[test]
    fn hand_requires_enough_jokers_to_fill_gaps() {
        let mut hand = Hand::new("player");
        let cards = [
            (Rank::Two, Suit::Diamonds),
            (Rank::Four, Suit::Clubs),
            (Rank::Six, Suit::Spades),
        ];
        for (rank, suit) in cards {
            hand.cards
                .push(Card::new_card(StandardCard::new_card(rank, suit)));
        }
        hand.cards.push(Card::new_card(StandardCard::new_card(
            Rank::Joker,
            Suit::Wild,
        )));

        assert!(hand.find_n_straight(4).is_none());
    }

    #[test]
    fn hand_finds_three_of_a_kind_using_joker() {
        let mut hand = Hand::new("player");
        let cards = [
            (Rank::Ten, Suit::Hearts),
            (Rank::Ten, Suit::Clubs),
            (Rank::Joker, Suit::Wild),
        ];
        for (rank, suit) in cards {
            hand.cards
                .push(Card::new_card(StandardCard::new_card(rank, suit)));
        }

        let trio = hand
            .find_n_of_a_kind(3)
            .expect("expected three of a kind with joker support");
        assert_eq!(trio.len(), 3);
        assert_eq!(trio.iter().filter(|card| card.rank == Rank::Ten).count(), 2);
        assert_eq!(
            trio.iter().filter(|card| card.rank == Rank::Joker).count(),
            1
        );
    }

    #[test]
    fn hand_allows_flush_with_joker() {
        let mut hand = Hand::new("player");
        let cards = [
            (Rank::Two, Suit::Hearts),
            (Rank::Four, Suit::Hearts),
            (Rank::Six, Suit::Hearts),
            (Rank::Nine, Suit::Hearts),
            (Rank::Joker, Suit::Wild),
        ];
        for (rank, suit) in cards {
            hand.cards
                .push(Card::new_card(StandardCard::new_card(rank, suit)));
        }

        assert!(hand.is_flush());
    }

    #[test]
    fn hand_of_only_jokers_counts_as_flush_and_of_a_kind() {
        let mut hand = Hand::new("player");
        for _ in 0..3 {
            hand.cards.push(Card::new_card(StandardCard::new_card(
                Rank::Joker,
                Suit::Wild,
            )));
        }

        assert!(hand.is_flush());
        let wild_trio = hand
            .find_n_of_a_kind(3)
            .expect("expected jokers to satisfy kind");
        assert_eq!(wild_trio.len(), 3);
        assert!(wild_trio.iter().all(|card| card.rank == Rank::Joker));
    }
}
