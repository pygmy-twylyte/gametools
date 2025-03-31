//! # Cards
//! 
//! This module implements mechanics common to games played with a standard 52 playing-card deck,
//! such as cards, decks, piles, and hands. Would perhaps one day like to add the capability to 
//! handle other sorts of cards (Uno, Old Maid, Memory, etc.).

// strum crate allows up to easily iterate through enums -- makes deck creation easy
use rand::seq::SliceRandom;
use std::fmt::Display;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{GameError, GameResult};

/// Represents all possible ranks (face values) for a standard playing card.
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

/// Represents the four possible suits of a standard playing card.
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

/// Traits common to any collection from which you can remove a card or cards. 
pub trait DrawFrom {
    fn draw(&mut self) -> Option<Card>;
    fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>>;
    fn size(&self) -> usize;
    fn name(&self) -> String;   // needed for GameError report when empty
}

/// A standard playing card.
/// 
/// New cards for use in gameplay should only be created by constructing a Deck. They will have a uid < u32::MAX
/// and always start face down. Temporary cards for use in comparisons and searches can be created individually,
/// but will always have a uid of u32::MAX and start face up.
/// 
/// ```rust
/// use gametools::Card;
/// use gametools::Rank::*;
/// use gametools::Suit::*;
/// 
/// let search_card = Card::new_temp(Queen, Spades);
/// assert_eq!(search_card.uid, u32::MAX);
/// assert!(search_card.face_up, "temporary cards have no need to be hidden from view");
/// assert_eq!(search_card.rank, Queen);
/// assert_eq!(search_card.suit, Spades);
/// 
/// if search_card.uid == u32::MAX {
///     println!("You created a temporary {search_card}.");
/// } else {
///     println!("Oops! This is a playable {search_card} from a deck!");
/// }
/// 
/// ```
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
    pub uid: u32,
    pub face_up: bool,
}
impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} of {}", self.rank.as_str(), self.suit.as_str())
    }
}
impl PartialEq for Card {
    /// Determines whether this card is the same as another, using only rank and suit
    /// (ignoring metadata like uid and face_up status)
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.suit == other.suit 
    }
}
impl Card {
    /// Creates a new temporary card for search / comparison purposes. The uid
    /// can be used to distinguish it from a card from a deck that actually belongs in play. 
    /// 
    /// ```rust
    /// use gametools::{Card, Rank, Suit};
    /// 
    /// let search_card = Card::new_temp( Rank::Queen, Suit::Spades );
    /// assert!(search_card.uid == u32::MAX, "oops! cards with uid < u32::MAX are playable")
    /// ```
    pub fn new_temp(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit, uid: u32::MAX, face_up: true }
    }
}

/// A deck of playing cards. A card source.
///
/// Cards can be only *removed* from a deck until it is empty. If more cards
/// are needed, a new deck must be created. This is unlike a Pile, to which
/// cards can also be added.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Deck {
    pub name: String,
    cards: Vec<Card>,
}
impl DrawFrom for Deck {
    /// Takes a card from the deck.
    /// 
    /// ```rust
    /// use gametools::{Deck, DrawFrom};
    /// 
    /// let mut deck = Deck::standard_52("main deck");
    /// 
    /// let drawn = deck.draw().unwrap();    // OK here, new deck always full
    /// println!("you drew: {drawn}");
    /// ```
    fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Takes multiple cards from the deck. Returns None if the deck doesn't have enough to fill the request.
    /// 
    /// ```rust
    /// use gametools::{Deck, DrawFrom};
    /// 
    /// let mut deck = Deck::standard_52("test_deck");
    /// 
    /// let three_cards = deck.draw_cards(3).unwrap();  // OK, new deck has > 3 cards here
    /// assert_eq!(deck.size(), 52 - 3);
    /// assert_eq!(three_cards.len(), 3);
    /// 
    /// for card in three_cards {
    ///     println!("{card}");
    /// }
    /// ```
    fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>> {
        if count > self.cards.len() {
            return None;
        }
        Some(self.cards.split_off(self.cards.len() - count))
    }
    
    /// Returns a string containing the deck's name for error reporting and display purposes
    fn name(&self) -> String {
        self.name.clone()
    }
    
    fn size(&self) -> usize {
        self.cards.len()
    }
}
impl Deck {
    /// Creates a new, standard 52-card deck of playing cards.
    /// 
    /// ```rust
    /// use gametools::{Deck,Card,DrawFrom};
    /// use gametools::Rank::*;
    /// use gametools::Suit::*;
    /// 
    /// let deck = Deck::standard_52("standard playing cards");
    /// 
    /// let size = deck.size();
    /// assert_eq!(size, 52);
    /// 
    /// let count_fives = deck.iter().filter(|&card| card.rank == Five).count();
    /// assert_eq!(count_fives, 4);
    /// 
    /// let count_qos = deck.iter()
    ///                     .filter(|&card| matches!(*card, Card{ rank: Queen, suit: Spades, ..}))
    ///                     .count();
    /// assert_eq!(count_qos, 1);
    /// ```
    pub fn standard_52(name: &str) -> Self {
        let mut cards = Vec::<Card>::new();
        let mut uid = 0;
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                cards.push(Card { rank, suit, uid, face_up: false });
                uid += 1;
            }
        }
        Self {
            name: name.to_owned(),
            cards,
        }
    }

    /// Provides an iterator over references to the cards remaining in the deck.
    pub fn iter(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter()
    }

    /// Provides a slice of references to the cards remaining in the deck.
    pub fn as_slice(&self) -> &[Card] {
        &self.cards
    }

    /// ```rust
    /// use gametools::{Deck, DrawFrom};
    /// 
    /// let mut deck = Deck::standard_52("deck_id");
    /// let original = deck.clone();
    /// assert_eq!(deck, original);
    /// deck.shuffle();
    /// assert_eq!(deck.size(), original.size());
    /// assert_ne!(deck, original);
    /// ```
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }

    /// Deals a specified number of cards to one or more hands.
    /// Returns a GameError if the deck doesn't contain enough cards to complete request for *all* hands.
    /// 
    /// ```rust
    /// use gametools::{Deck, Hand, DrawFrom};
    /// 
    /// // create game deck
    /// let mut war_deck = Deck::standard_52("War!");
    /// war_deck.shuffle();
    /// 
    /// // create (empty) hands for the players
    /// let mut player_1 = Hand::new("Frank");
    /// let mut player_2 = Hand::new("Dweezil");
    /// let mut hands = vec![player_1, player_2];
    /// 
    /// // deal 26 cards each to Frank and Dweezil
    /// war_deck.deal_to_hands(&mut hands, 26).unwrap();
    /// 
    /// assert_eq!(war_deck.size(), 0);
    /// assert_eq!(hands[0].size(), 26);
    /// assert_eq!(hands[1].size(), 26);
    /// 
    /// ```
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

/// A stack of cards, such as a draw or discard pile. Last one added is first that will be drawn. 
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
    
    fn size(&self) -> usize {
        self.cards.len()
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

    pub fn size(&self) -> usize {
        self.cards.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::cards::*;

    #[test]
    fn create_standard_deck_works() {
        let deck = Deck::standard_52("Standard/Test Deck");

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
        let mut deck = Deck::standard_52("test deck");
        let Card { rank, suit, .. } = deck
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
        let mut deck = Deck::standard_52("test deck");
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
        let mut deck = Deck::standard_52("standard 52-card deck");

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
        let mut deck = Deck::standard_52("standard test deck");

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
        let mut deck = Deck::standard_52("the poodle bites");

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
