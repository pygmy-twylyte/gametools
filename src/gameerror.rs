use std::error::Error;
use std::fmt;

/// Error types for problematic game conditions.
#[derive(Debug)]
pub enum GameError {
    StackEmpty(String),
    StackTooSmall(String),
    CardNotFound,
    InsufficientTiles,
    TileUnconnected,
    TrainClosed,
}
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::StackEmpty(n) => write!(f, "cannot draw from empty stack '{n}'"),
            GameError::StackTooSmall(n) => {
                write!(f, "too few cards remain in '{n}' to satisfy need")
            }
            GameError::CardNotFound => {
                write!(f, "the card sought was not found in this collection")
            }
            GameError::InsufficientTiles => {
                write!(f, "insufficient tiles left in the bone pile")
            }
            GameError::TileUnconnected => {
                write!(f, "that tile does not match the tail of the train")
            }
            GameError::TrainClosed => {
                write!(f, "attempted to play on a closed train")
            }
        }
    }
}
impl Error for GameError {}
