// strum crate allows up to easily iterate through enums -- makes deck creation easy
use rand::seq::SliceRandom;
use std::fmt::Display;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{GameError, GameResult};

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

/// Any collection of cards that you can draw 1 or more from
pub trait DrawFrom {
    fn draw(&mut self) -> Option<Card>;
    fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>>;
    fn name(&self) -> String;   // needed for GameError report when empty
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

/// A deck of playing cards. A card source.
///
/// Cards can only be removed from a deck until it is empty. If more cards
/// are needed, a new deck must be created. This is unlike a Pile, to which
/// cards can also be added.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Deck {
    pub name: String,
    pub cards: Vec<Card>,
}
impl DrawFrom for Deck {
    /// Takes a card from the deck.
    fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Takes multiple cards from the deck. Returns None if the deck doesn't have enough to fill the request.
    fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>> {
        if count > self.cards.len() {
            return None;
        }
        Some(self.cards.split_off(self.cards.len() - count))
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
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

    /// Shuffles the cards in the deck in place.
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    /// Deals a specified number of cards to one or more hands.
    /// Returns Err() if the deck doesn't contain enough cards.
    pub fn deal_to_hands(
        &mut self,
        hands: &mut Vec<Hand>,
        count: usize,
    ) -> GameResult<()> {
        // return Err immediately if there aren't enough cards left, so we don't
        // have to partially fill hands before finding the end of the deck
        if hands.len() * count > self.cards.len() {
            return Err(GameError::StackTooSmall(self.name()));
        }

        for hand in hands {
            hand.draw_cards_from(self, count)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pile {
    pub name: String,
    pub cards: Vec<Card>,
}
impl DrawFrom for Pile {
    fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>> {
        if count > self.cards.len() {
            return None;
        }
        Some(self.cards.split_off(self.cards.len() - count))
    }
    
    fn name(&self) -> String {
        self.name.clone()
    }
}
impl Pile {
    /// Add a card to the top of the pile.
    pub fn add(&mut self, card: Card) {
        self.cards.push(card);
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

    /// Draws a card from the specified Deck or Pile.
    /// Returns Err if deck is empty.
    pub fn draw_card_from(&mut self, stack: &mut impl DrawFrom) -> GameResult<()> {
        let drawn = match stack.draw() {
            Some(card) => card,
            None => return Err(GameError::StackEmpty(stack.name())),
        };
        self.cards.push(drawn);
        Ok(())
    }

    /// Draws a specified number of cards from a Deck or a Pile
    /// Returns Err() if there aren't enough cards to fulfill the request.
    pub fn draw_cards_from(
        &mut self,
        stack: &mut impl DrawFrom,
        count: usize,
    ) -> GameResult<()> {
        let mut drawn = match stack.draw_cards(count) {
            Some(cards) => cards,
            None => return Err(GameError::StackTooSmall(stack.name())),
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
    fn deck_deal_to_hands_works() -> Result<(), Box<dyn std::error::Error>> {
        let mut deck = Deck::new("standard 52-card deck");

        // create a pool of empty hands
        let num_hands = 4;
        let num_cards = 5;
        let mut hands = Vec::<Hand>::new();
        for n in 1..=num_hands {
            hands.push(Hand::new(&format!("Player {n}")));
        }

        // deal 'em
        deck.deal_to_hands(&mut hands, num_cards)?;

        //dbg!(&hands); // uncomment to show all hands after deal
        assert_eq!(deck.cards.len(), 52 - (num_cards * num_hands));
        assert_eq!(hands.len(), num_hands);

        // do all hands have the right # of cards?
        for hand in &hands {
            assert_eq!(hand.cards.len(), num_cards);
        }

        // should Err if we request too many
        let too_many = deck.deal_to_hands(&mut hands, 100);
        assert!(
            matches!(too_many, Err(_)),
            "too-many request should have returned Err from deal_to_hands"
        );

        Ok(())
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
