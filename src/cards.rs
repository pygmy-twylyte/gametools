//! # Cards Module
//!
//! This module implements mechanics common to games played with a standard 52 playing-card deck,
//! such as cards, decks, piles, and hands. Would perhaps one day like to add the capability to
//! handle other sorts of cards (Uno, Old Maid, Memory, etc.).
//!
//! # Example
//! ```
//! use gametools::{Deck, CardHand, Pile, AddCard, TakeCard};
//! use gametools::{Suit, Rank, Card};
//! use gametools::{GameResult};
//!
//! fn main() -> GameResult<()> {
//!    let mut deck = Deck::standard_52("main deck");
//!    let mut hand = CardHand::new("player_1");
//!    let mut discard_pile = Pile::new("discard");
//!
//!    // shuffle and draw 7 cards into the hand
//!    deck.shuffle();
//!    hand.draw_cards_from(&mut deck, 7)?;
//!    println!("{hand}");
//!
//!    // or deal from the deck to multiple hands
//!    let mut other_hands = vec![CardHand::new("player_2"), CardHand::new("player_3")];
//!    deck.deal_to_hands(&mut other_hands, 7)?;
//!
//!    // count ranks and suits
//!    let num_spades = hand.count_suit(Suit::Spades);
//!    let num_queens = hand.count_rank(Rank::Queen);
//!
//!    // search for a card in a hand
//!    let search_card = Card::new_temp(Rank::Ace, Suit::Clubs);
//!    let go_fish = hand.contains(&search_card);
//!
//!    // move a card from hand to another hand or a pile; returns an error if card is not in hand
//!    if let Err(e) = hand.transfer_card(&search_card,&mut discard_pile) {
//!         println!("error: {e}")
//!    };
//!
//!    Ok(())
//! }
//!
//! ```

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
    /// Returns static string representation of each variant.
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

/// Trait common to any collection from which you can remove a card or cards.
pub trait TakeCard {
    fn draw(&mut self) -> Option<Card>;
    fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>>;
    fn size(&self) -> usize;
    fn name(&self) -> String; // needed for GameError report when empty
}

/// Trait common to any collection to which you can add cards.
pub trait AddCard {
    /// Adds a single card to the collection.
    fn add_card(&mut self, card: Card);
    /// Adds multiple cards to the collection.
    fn add_cards(&mut self, cards: &mut Vec<Card>);
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
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
    pub uid: u32,
    pub face_up: bool,
}
impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} of {}]", self.rank.as_str(), self.suit.as_str())
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
        Self {
            rank,
            suit,
            uid: u32::MAX,
            face_up: true,
        }
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
impl TakeCard for Deck {
    /// Takes a card from the deck.
    ///
    /// ```rust
    /// use gametools::{Deck, TakeCard};
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
    /// use gametools::{Deck, TakeCard};
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
    /// use gametools::{Deck,Card,TakeCard};
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
                cards.push(Card {
                    rank,
                    suit,
                    uid,
                    face_up: false,
                });
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

    /// Shuffles the deck in place.
    /// ```rust
    /// use gametools::{Deck, TakeCard};
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
    /// use gametools::{Deck, CardHand, TakeCard};
    ///
    /// // create game deck
    /// let mut war_deck = Deck::standard_52("War!");
    /// war_deck.shuffle();
    ///
    /// // create (empty) hands for the players
    /// let mut player_1 = CardHand::new("Frank");
    /// let mut player_2 = CardHand::new("Dweezil");
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
    pub fn deal_to_hands(&mut self, hands: &mut Vec<CardHand>, count: usize) -> GameResult<()> {
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
impl TakeCard for Pile {
    fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    fn draw_cards(&mut self, count: usize) -> Option<Vec<Card>> {
        if count > self.cards.len() {
            return None;
        }
        Some(self.cards.split_off(self.cards.len() - count))
    }
    /// Returns the name of the pile.
    fn name(&self) -> String {
        self.name.clone()
    }
    /// Returns the size of the pile (in cards).
    fn size(&self) -> usize {
        self.cards.len()
    }
}
impl Pile {
    /// Create a new empty pile of cards.
    ///
    /// ```
    /// use gametools::{Pile, TakeCard};
    /// let mut discard_pile = Pile::new("Discard");
    /// assert_eq!(discard_pile.size(), 0);
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            cards: Vec::<Card>::new(),
        }
    }
}
impl AddCard for Pile {
    /// Add a card to the top of the pile.
    /// ```
    /// # use gametools::{Pile, Card, Deck, TakeCard, AddCard, GameResult};
    /// # fn main() -> GameResult<()> {
    ///     let mut pile = Pile::new("Discard");
    ///     let mut deck = Deck::standard_52("Game Deck");
    ///     deck.shuffle();
    ///
    ///     let card = deck.draw().unwrap();
    ///     pile.add_card(card);
    ///
    ///     assert_eq!(pile.size(), 1);
    ///     assert_eq!(deck.size(), 51);
    /// # Ok(())
    /// }
    /// ```
    fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn add_cards(&mut self, cards: &mut Vec<Card>) {
        self.cards.append(cards);
    }
}

/// A player's hand of cards in a game.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CardHand {
    pub player: String,
    pub cards: Vec<Card>,
}

impl Display for CardHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut display_string = format!("{}:[", self.player);
        for card in &self.cards {
            display_string.push_str(&card.to_string());
        }
        display_string.push(']');
        write!(f, "{}", display_string)
    }
}
impl AddCard for CardHand {
    fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn add_cards(&mut self, cards: &mut Vec<Card>) {
        self.cards.append(cards);
    }
}

impl CardHand {
    /// Takes a player name or ID and returns a new empty hand.
    pub fn new(player: &str) -> Self {
        Self {
            player: player.to_string(),
            cards: Vec::<Card>::new(),
        }
    }

    /// Draws a card from the specified Deck or Pile.
    /// Returns Err if deck is empty.
    pub fn draw_card_from(&mut self, stack: &mut impl TakeCard) -> GameResult<()> {
        let drawn = match stack.draw() {
            Some(card) => card,
            None => return Err(GameError::StackEmpty(stack.name())),
        };
        self.cards.push(drawn);
        Ok(())
    }

    /// Draws a specified number of cards from a Deck or a Pile
    /// Returns Err() if there aren't enough cards to fulfill the request.
    pub fn draw_cards_from(&mut self, stack: &mut impl TakeCard, count: usize) -> GameResult<()> {
        let mut drawn = match stack.draw_cards(count) {
            Some(cards) => cards,
            None => return Err(GameError::StackTooSmall(stack.name())),
        };
        self.cards.append(&mut drawn);
        Ok(())
    }

    /// Returns the number of cards currently in the hand.
    pub fn size(&self) -> usize {
        self.cards.len()
    }

    /// Returns true if a specified card is found in the hand.
    pub fn contains(&self, temp_card: &Card) -> bool {
        self.cards.contains(temp_card)
    }

    /// Returns true if a card of specified Rank and Suit is in the hand.
    pub fn contains_by_rs(&self, rank: Rank, suit: Suit) -> bool {
        self.cards.contains(&Card::new_temp(rank, suit))
    }

    /// Transfer a card from this hand to another collection.
    ///
    /// The other collection can be a Pile, a Hand or anything that implements AddCard.
    /// Returns a GameError::CardNotFound if you try to transfer a card that isn't there.
    /// ```
    /// # use gametools::*;
    /// # fn main() -> GameResult<()> {
    /// let mut deck = Deck::standard_52("test");
    /// let mut hand = CardHand::new("player 1");
    /// let mut other = CardHand::new("player 2");
    ///
    /// // Since we haven't shuffled the deck, this will put the
    /// // Ace, King, and Queen of Hearts into the hand
    /// hand.draw_cards_from(&mut deck, 3)?;
    ///
    /// // Transfer the Ace into the other hand
    /// let search_card = Card::new_temp(Rank::Ace, Suit::Hearts);
    /// hand.transfer_card(&search_card, &mut other)?;
    ///
    /// assert_eq!(deck.size(), 49);
    /// assert_eq!(hand.size(), 2);
    /// assert!(other.contains(&search_card));
    /// assert!(!hand.contains(&search_card));
    ///
    /// # Ok(())
    /// # }
    ///
    /// ```
    pub fn transfer_card(&mut self, temp_card: &Card, other: &mut impl AddCard) -> GameResult<()> {
        match self.cards.iter().position(|&c| c == *temp_card) {
            Some(pos) => {
                let real_card = self.cards.remove(pos);
                other.add_card(real_card);
            }
            None => return Err(GameError::CardNotFound),
        }
        Ok(())
    }

    /// Count the cards in the hand matching a given rank.
    pub fn count_rank(&self, rank: Rank) -> usize {
        self.cards.iter().filter(|&c| c.rank == rank).count()
    }

    /// Count the cards in the hand matching a given suit.
    pub fn count_suit(&self, suit: Suit) -> usize {
        self.cards.iter().filter(|&c| c.suit == suit).count()
    }
}

#[cfg(test)]
mod card_tests {
    use crate::cards::*;

    #[test]
    fn rank_as_str_works() {
        assert_eq!(Rank::Two.as_str(), "Two");
        assert_eq!(Rank::Three.as_str(), "Three");
        assert_eq!(Rank::Four.as_str(), "Four");
        assert_eq!(Rank::Five.as_str(), "Five");
        assert_eq!(Rank::Six.as_str(), "Six");
        assert_eq!(Rank::Seven.as_str(), "Seven");
        assert_eq!(Rank::Eight.as_str(), "Eight");
        assert_eq!(Rank::Nine.as_str(), "Nine");
        assert_eq!(Rank::Ten.as_str(), "Ten");
        assert_eq!(Rank::Jack.as_str(), "Jack");
        assert_eq!(Rank::Queen.as_str(), "Queen");
        assert_eq!(Rank::King.as_str(), "King");
        assert_eq!(Rank::Ace.as_str(), "Ace");
    }

    #[test]
    fn suit_as_str_works() {
        assert_eq!(Suit::Clubs.as_str(), "Clubs");
        assert_eq!(Suit::Diamonds.as_str(), "Diamonds");
        assert_eq!(Suit::Hearts.as_str(), "Hearts");
        assert_eq!(Suit::Spades.as_str(), "Spades");
    }

    #[test]
    fn card_display_is_correct() {
        let qos = Card::new_temp(Rank::Queen,Suit::Spades);
        assert_eq!(qos.to_string(), "[Queen of Spades]".to_string());
    }

    #[test]
    fn deck_iter_works() {
        let deck = Deck::standard_52("test");
        assert_eq!(deck.iter().count(), 52);
    }

    #[test]
    fn deck_as_slice_works() {
        let deck = Deck::standard_52("test");
        for card in deck.as_slice() {
            assert!(card.uid < u32::MAX);
        }
        assert_eq!(deck.as_slice().iter().count(), 52);
    }

    #[test]
    fn deck_shuffle_works() {
        let original = Deck::standard_52("test");
        let mut copy = original.clone();
        assert_eq!(original, copy);
        copy.shuffle();
        assert_eq!(original.size(), copy.size());
        assert_ne!(original, copy);
    }

    #[test]
    fn deck_standard_52_works() {
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
        let mut hands = Vec::<CardHand>::new();
        for n in 1..=num_hands {
            hands.push(CardHand::new(&format!("Player {n}")));
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
    fn pile_add_card_works() {
        let mut pile = Pile::new("p1");
        pile.add_card(Card::new_temp(Rank::Ace, Suit::Clubs));
        assert_eq!(pile.size(), 1);
    }

    #[test]
    fn pile_add_cards_works() {
        let mut pile = Pile::new("p1");
        let mut some_cards = vec![
            Card::new_temp(Rank::King, Suit::Diamonds),
            Card::new_temp(Rank::Ten, Suit::Hearts),
            Card::new_temp(Rank::Queen, Suit::Spades),
        ];
        pile.add_cards(&mut some_cards);
        assert_eq!(pile.size(), 3);
    }

    #[test]
    fn pile_draw_works() {
        let mut pile = Pile::new("discard");
        let mut some_cards = vec![
            Card::new_temp(Rank::King, Suit::Diamonds),
            Card::new_temp(Rank::Ten, Suit::Hearts),
            Card::new_temp(Rank::Queen, Suit::Spades),
        ];
        pile.add_cards(&mut some_cards);

        // top card should be the last one added
        let top_card = pile.draw().unwrap();
        assert_eq!(top_card, Card::new_temp(Rank::Queen, Suit::Spades));

        // draw from an empty pile should return None
        let mut empty_pile = Pile::new("test");
        assert!(empty_pile.draw().is_none());
    }

    #[test]
    fn pile_draw_cards_works() {
        let mut pile = Pile::new("discard");
        let mut some_cards = vec![
            Card::new_temp(Rank::King, Suit::Diamonds),
            Card::new_temp(Rank::Ten, Suit::Hearts),
            Card::new_temp(Rank::Queen, Suit::Spades),
        ];
        pile.add_cards(&mut some_cards);
        let take_two = pile.draw_cards(2).unwrap();
        let expected = vec![
            Card::new_temp(Rank::Ten, Suit::Hearts),
            Card::new_temp(Rank::Queen, Suit::Spades),
        ];
        assert_eq!(take_two, expected);

        // attempt to draw too many should return None
        assert!(pile.draw_cards(1000).is_none());
    }

    #[test]
    fn pile_name_works() {
        let pile = Pile::new("test");
        assert_eq!(pile.name(), "test");
    }

    #[test]
    fn pile_size_is_correct() {
        let mut pile = Pile::new("test");
        assert_eq!(pile.size(),  0);
        pile.add_card(Card::new_temp(Rank::Ace, Suit::Hearts));
        assert_eq!(pile.size(), 1);
    }

    #[test]
    fn hand_display_is_correct() {
        let mut hand = CardHand::new("test");
        assert_eq!(hand.to_string(), "test:[]");
        let qos = Card::new_temp(Rank::Queen, Suit::Spades);
        hand.add_card(qos);
        assert_eq!(hand.to_string(), "test:[[Queen of Spades]]");
    }

    #[test]
    fn hand_draw_card_from_works() -> Result<(), Box<dyn std::error::Error>> {
        let mut hand = CardHand::new("Player 1");
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
        let mut hand = CardHand::new("frank zappa");
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

    #[test]
    fn hand_transfer_card_to_pile_works() -> GameResult<()> {
        let mut deck = Deck::standard_52("test deck");
        let mut pile = Pile::new("test pile");
        let mut hand = CardHand::new("test hand");

        // Since we haven't shuffled the deck, this will put the
        // Ace, King, and Queen of Hearts into the hand
        hand.draw_cards_from(&mut deck, 3)
            .expect("deck just created should have >3 cards!");

        // Transfer a card we know is in the hand to the pile
        let search_card = Card::new_temp(Rank::Queen, Suit::Hearts);
        hand.transfer_card(&search_card, &mut pile)?;
        assert_eq!(pile.size(), 1);
        assert_eq!(hand.size(), 2);

        // Should return Err if we try to transfer a card that isn't there
        let search_card = Card::new_temp(Rank::Queen, Suit::Spades);
        assert!(
            hand.transfer_card(&search_card, &mut pile).is_err(),
            "should not be able to transfer an absent card"
        );
        Ok(())
    }
    #[test]
    fn hand_transfer_card_to_hand_works() -> GameResult<()> {
        let mut deck = Deck::standard_52("test deck");
        let mut hand = CardHand::new("test hand");
        let mut other = CardHand::new("other hand");

        // Since we haven't shuffled the deck, this will put the
        // Ace, King, and Queen of Hearts into the hand
        hand.draw_cards_from(&mut deck, 3)
            .expect("deck just created should have >3 cards!");

        // Transfer a card we know is in the hand to the other hand
        let search_card = Card::new_temp(Rank::Queen, Suit::Hearts);
        hand.transfer_card(&search_card, &mut other)?;
        assert_eq!(other.size(), 1);
        assert_eq!(hand.size(), 2);

        // Should return Err if we try to transfer a card that isn't there
        let search_card = Card::new_temp(Rank::Queen, Suit::Spades);
        assert!(
            hand.transfer_card(&search_card, &mut other).is_err(),
            "should not be able to transfer an absent card"
        );
        Ok(())
    }
    #[test]
    fn hand_count_rank_works() {
        let mut hand = CardHand::new("p1");
        hand.add_card(Card::new_temp(Rank::Queen, Suit::Spades));
        hand.add_card(Card::new_temp(Rank::Queen, Suit::Clubs));
        hand.add_card(Card::new_temp(Rank::Three, Suit::Spades));
        assert_eq!(hand.count_rank(Rank::Queen), 2);
        assert_eq!(hand.count_rank(Rank::Three), 1);
        assert_eq!(hand.count_rank(Rank::Jack), 0);
    }

    #[test]
    fn hand_count_suit_works() {
        let mut hand = CardHand::new("p1");
        hand.add_card(Card::new_temp(Rank::Queen, Suit::Spades));
        hand.add_card(Card::new_temp(Rank::Queen, Suit::Clubs));
        hand.add_card(Card::new_temp(Rank::Three, Suit::Spades));
        assert_eq!(hand.count_suit(Suit::Clubs), 1);
        assert_eq!(hand.count_suit(Suit::Spades), 2);
        assert_eq!(hand.count_suit(Suit::Hearts), 0)
    }
    #[test]
    fn hand_contains_works() {
        let mut hand = CardHand::new("test");
        hand.add_card(Card::new_temp(Rank::Queen, Suit::Spades));

        assert!(hand.contains(&Card::new_temp(Rank::Queen, Suit::Spades)));
        assert!(!hand.contains(&Card::new_temp(Rank::Ten, Suit::Clubs)));
    }

    #[test]
    fn hand_contains_rs_works() {
        let mut hand = CardHand::new("test");
        let temp_card = Card::new_temp(Rank::Ace, Suit::Diamonds);
        hand.add_card(temp_card);

        assert!(hand.contains_by_rs(Rank::Ace, Suit::Diamonds));
        assert!(!hand.contains_by_rs(Rank::Jack, Suit::Clubs));
    }

    #[test]
    fn hand_add_card_works() {
        let mut hand = CardHand::new("p1");
        hand.add_card(Card::new_temp(Rank::Ace, Suit::Clubs));
        assert_eq!(hand.size(), 1);
    }

    #[test]
    fn hand_add_cards_works() {
        let mut hand = CardHand::new("p1");
        let mut some_cards = vec![
            Card::new_temp(Rank::King, Suit::Diamonds),
            Card::new_temp(Rank::Ten, Suit::Hearts),
            Card::new_temp(Rank::Queen, Suit::Spades),
        ];
        hand.add_cards(&mut some_cards);
        assert_eq!(hand.size(), 3);
    }
}
