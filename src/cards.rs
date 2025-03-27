// strum crate allows up to easily iterate through enums -- makes deck creation easy
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
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, EnumIter)]
pub enum Suit {
    Clubs,
    Diamonds,
    Spades,
    Hearts,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

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
    fn draw_works() {
        let mut deck = Deck::new("test deck");
        let Card{rank, suit} = deck.draw().expect("should be able to draw from new full deck");
        let remaining_of_rank = deck.cards.iter().filter(|&c| c.rank == rank).count();
        let remaining_of_suit = deck.cards.iter().filter(|&c| c.suit == suit).count();

        assert_eq!(deck.cards.len(), 51);
        assert_eq!(remaining_of_rank, 3);
        assert_eq!(remaining_of_suit, 12);
    }

    #[test]
    fn draw_cards_works() {
        let mut deck = Deck::new("test deck");
        let hand = deck.draw_cards(5).expect("should be able to draw 5 from fresh deck");
        assert_eq!(hand.len(), 5);
        assert_eq!(deck.cards.len(), 47);

        let huge_hand = deck.draw_cards(100);
        assert_eq!(huge_hand, None)
    }
}
