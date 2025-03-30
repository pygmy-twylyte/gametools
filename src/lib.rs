mod dice;
pub use dice::{Die, DicePool};

mod cards;
pub use cards::{Card, Deck, Pile, Hand, DrawFrom};

mod gameerror;
pub use gameerror::GameError;

pub type GameResult<T> = Result<T, GameError>;



