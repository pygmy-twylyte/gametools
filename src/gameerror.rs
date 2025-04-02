use std::fmt;
use std::error::Error;

/// Error types for problematic game conditions. 
#[derive(Debug)]
pub enum GameError {
    StackEmpty(String),
    StackTooSmall(String),
    CardNotFound,
}
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::StackEmpty(n) => write!(f, "cannot draw from empty stack '{n}'"),
            GameError::StackTooSmall(n) => write!(f, "too few cards remain in '{n}' to satisfy need"),
            GameError::CardNotFound => write!(f, "the card sought was not found in this collection"),
        }
    }
}
impl Error for GameError {}
