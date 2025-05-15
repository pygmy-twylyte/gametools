//! # Game Error Module
//!
//! This module defines the [`GameError`] enum, which represents common error conditions
//! that may arise during gameplay logic—such as trying to draw from an empty stack,
//! attempting to play a tile that doesn’t match, or encountering missing components.
//!
//! These errors implement [`std::error::Error`] and [`std::fmt::Display`], allowing them
//! to integrate cleanly with Rust's error handling patterns.
use std::error::Error;
use std::fmt;

/// Error types for problematic game conditions.
#[derive(Debug, PartialEq)]
pub enum GameError {
    StackEmpty(String),
    StackTooSmall(String),
    CardNotFound,
    InsufficientTiles,
    TileUnconnected,
    TrainClosed,
    SpinnerEmpty,
    DicePoolWithNoDice,
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
            GameError::SpinnerEmpty => {
                write!(
                    f,
                    "spin() returned None: empty spinner or landed on covered wedge"
                )
            }
            GameError::DicePoolWithNoDice => {
                write!(f, "attempted to roll zero dice into a DicePool")
            }
        }
    }
}
impl Error for GameError {}

#[cfg(test)]
mod tests {
    use super::GameError;
    use std::error::Error;

    #[test]
    fn test_game_error_display_and_trait() {
        let cases: Vec<(GameError, &str)> = vec![
            (
                GameError::StackEmpty("Main".to_string()),
                "cannot draw from empty stack 'Main'",
            ),
            (
                GameError::StackTooSmall("Reserve".to_string()),
                "too few cards remain in 'Reserve' to satisfy need",
            ),
            (
                GameError::CardNotFound,
                "the card sought was not found in this collection",
            ),
            (
                GameError::InsufficientTiles,
                "insufficient tiles left in the bone pile",
            ),
            (
                GameError::TileUnconnected,
                "that tile does not match the tail of the train",
            ),
            (
                GameError::TrainClosed,
                "attempted to play on a closed train",
            ),
            (
                GameError::SpinnerEmpty,
                "spin() returned None: empty spinner or landed on covered wedge",
            ),
            (
                GameError::DicePoolWithNoDice,
                "attempted to roll zero dice into a DicePool",
            ),
        ];

        for (err, expected_msg) in cases {
            assert_eq!(err.to_string(), expected_msg);

            // Confirm it behaves as std::error::Error
            let as_error: &dyn Error = &err;
            assert_eq!(as_error.to_string(), expected_msg);
        }
    }
}
