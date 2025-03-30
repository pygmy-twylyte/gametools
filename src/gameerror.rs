use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum GameError {
    StackEmpty(String),
    StackTooSmall(String),
}
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::StackEmpty(n) => write!(f, "cannot draw from empty stack '{n}'"),
            GameError::StackTooSmall(n) => write!(f, "too few cards remain in '{n}' to satisfy need")
        }
    }
}
impl Error for GameError {}
