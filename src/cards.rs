// strum crate allows up to easily iterate through enums -- makes deck creation easy
use rand::seq::SliceRandom;
use std::fmt::Display;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, EnumIter)]
pub enum Rank {
    Two,
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
}
impl Rank {
    pub fn as_str(&self) -> &'static str {
        match &self {
            Rank::Two => "Two",
            Rank::Three => "Three",
            Rank::Four => "Four",
            Rank::Five => "Five",
            Rank::Six => "Six",
            Rank::Seven => "Seven",
            Rank::Eight => "Eight",
            Rank::Nine => "Nine",
            Rank::Ten => "Ten",
            Rank::Jack => "Jack",
            Rank::Queen => "Queen",
            Rank::King => "King",
            Rank::Ace => "Ace",
        }
    }
}
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, EnumIter)]
pub enum Suit {
    Clubs,
    Diamonds,
    Spades,
    Hearts,
}
impl Suit {
    pub fn as_str(&self) -> &'static str {
        match self {
            Suit::Clubs => "Clubs",
            Suit::Diamonds => "Diamonds",
            Suit::Spades => "Spades",
            Suit::Hearts => "Hearts",
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}
impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} of {})", self.rank.as_str(), self.suit.as_str())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Deck {
    pub cards: Vec<Card>,
    pub name: String,
}
impl Deck {
    /// Creates a new, standard 52-card deck of playing cards.
    pub fn new(name: &str) -> Self {
        let mut cards = Vec::<Card>::new();
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                cards.push(Card { rank, suit });
            }
        }
        Self {
            name: name.to_owned(),
            cards,
        }
    }

    /// Takes a card from the deck.
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Takes multiple cards from the deck. Returns None if the deck doesn't have enough to fill the request.
    pub fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>> {
        if count > self.cards.len() {
            return None;
        }

        Some(self.cards.drain(..count).collect())
    }

    /// Shuffles the cards in the deck in place.
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }
}

/// A player's hand of cards in a game.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Hand {
    pub player: String,
    pub cards: Vec<Card>,
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_string = format!("{}:[", self.player);
        for card in &self.cards {
            display_string.push_str(&card.to_string());
        }
        display_string.push(']');
        write!(f, "{}", display_string)
    }
}

impl Hand {
    /// Takes a player name or ID and returns a new empty hand.
    pub fn new(player: &str) -> Self {
        Self {
            player: player.to_string(),
            cards: Vec::<Card>::new(),
        }
    }

    /// Draws a card from the specified Deck.
    /// Returns Err if deck is empty.
    pub fn draw_card_from(&mut self, deck: &mut Deck) -> Result<(), &'static str> {
        let drawn = match deck.draw() {
            Some(card) => card,
            None => return Err("cannot draw a card: deck is empty"),
        };
        self.cards.push(drawn);
        Ok(())
    }

    /// Draws a specified number of cards from a Deck.
    /// Returns Err() if there aren't enough cards to fulfill the request.
    pub fn draw_cards_from(&mut self, deck: &mut Deck, count: usize) -> Result<(), &'static str> {
        let mut drawn = match deck.draw_cards(count) {
            Some(cards) => cards,
            None => return Err("not enough cards in deck for draw request"),
        };
        self.cards.append(&mut drawn);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::*;

    #[test]
    fn create_standard_deck_works() {
        let deck = Deck::new("Standard/Test Deck");

        let spade_count = deck
            .cards
            .iter()
            .filter(|&c| c.suit == Suit::Spades)
            .count();
        let jack_count = deck.cards.iter().filter(|&c| c.rank == Rank::Jack).count();
        let heart_count = deck
            .cards
            .iter()
            .filter(|&c| c.suit == Suit::Hearts)
            .count();
        let two_count = deck.cards.iter().filter(|&c| c.rank == Rank::Two).count();
        let all_count = deck.cards.len();

        assert_eq!(spade_count, 13);
        assert_eq!(jack_count, 4);
        assert_eq!(heart_count, 13);
        assert_eq!(two_count, 4);
        assert_eq!(all_count, 52);
    }

    #[test]
    fn deck_draw_works() {
        let mut deck = Deck::new("test deck");
        let Card { rank, suit } = deck
            .draw()
            .expect("should be able to draw from new full deck");
        let remaining_of_rank = deck.cards.iter().filter(|&c| c.rank == rank).count();
        let remaining_of_suit = deck.cards.iter().filter(|&c| c.suit == suit).count();

        assert_eq!(deck.cards.len(), 51);
        assert_eq!(remaining_of_rank, 3);
        assert_eq!(remaining_of_suit, 12);
    }

    #[test]
    fn deck_draw_cards_works() {
        let mut deck = Deck::new("test deck");
        let hand = deck
            .draw_cards(5)
            .expect("should be able to draw 5 from fresh deck");
        assert_eq!(hand.len(), 5);
        assert_eq!(deck.cards.len(), 47);

        let huge_hand = deck.draw_cards(100);
        assert_eq!(huge_hand, None)
    }

    #[test]
    fn hand_draw_card_from_works() -> Result<(), Box<dyn std::error::Error>> {
        let mut hand = Hand::new("Player 1");
        let mut deck = Deck::new("standard test deck");

        assert_eq!(hand.cards.len(), 0);
        assert_eq!(deck.cards.len(), 52);

        hand.draw_card_from(&mut deck)?;

        assert_eq!(hand.cards.len(), 1);
        assert_eq!(deck.cards.len(), 51);

        let _ = deck.draw_cards(51); // empty the deck and try to draw again
        let empty_draw = hand.draw_card_from(&mut deck);
        assert!(
            matches!(empty_draw, Err(_)),
            "draw from empty deck should have returned err"
        );

        Ok(())
    }

    #[test]
    fn hand_draw_cards_from_works() -> Result<(), Box<dyn std::error::Error>> {
        let mut hand = Hand::new("frank zappa");
        let mut deck = Deck::new("the poodle bites");

        hand.draw_cards_from(&mut deck, 5)?;
        assert_eq!(hand.cards.len(), 5);
        assert_eq!(deck.cards.len(), 47);

        let too_many = hand.draw_cards_from(&mut deck, 500);
        assert!(
            matches!(too_many, Err(_)),
            "attempt to draw too many cards should have returned Err()"
        );
        Ok(())
    }
}
