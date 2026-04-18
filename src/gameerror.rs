//! Shared error types used across the crate.
//!
//! [`GameError`] covers higher-level collection and gameplay failures, while [`DiceError`]
//! keeps the dice module's validation errors specific and can be converted into
//! [`GameError`] when needed.
//!

use thiserror::Error;

/// Error types for problematic game conditions.
#[derive(Debug, Error, PartialEq)]
pub enum GameError {
    #[error("cannot draw from empty stack '{0}'")]
    StackEmpty(String),
    #[error("too few cards remain in '{0}' to satisfy need")]
    StackTooSmall(String),
    #[error("the card sought was not found in this collection")]
    CardNotFound,
    #[error("insufficient tiles left in the bone pile")]
    InsufficientTiles,
    #[error("that tile does not match the tail of the train")]
    TileUnconnected,
    #[error("tile with id '{0}' not found in hand")]
    TileNotFound(usize),
    #[error("attempted to play on a closed train")]
    TrainClosed,
    #[error("spin() returned None: empty spinner or landed on covered wedge")]
    SpinnerEmpty,
    #[error("attempted to roll zero dice into a DicePool")]
    DicePoolWithNoDice,
    #[error("attempted to create a die with zero sides")]
    DieWithZeroSides,
    #[error("refilling pool must have items with which to refill")]
    PoolCannotBeEmpty,
    #[error("invalid index {0} for pool size {1}")]
    InvalidPoolIndex(usize, usize),
    #[error("dice error: {0}")]
    DiceError(#[from] DiceError),
}

/// Errors specific to creating and rolling dice.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum DiceError {
    #[error("a die with zero sides cannot be created")]
    DieWithNoSides,
    #[error("invalid explode trigger: {explode_on} on {sides}-sided die")]
    InvalidExplodeTrigger { explode_on: u64, sides: u64 },
    #[error("one sided die would infinitely explode")]
    InfiniteExplosion,
}

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
            (
                GameError::DieWithZeroSides,
                "attempted to create a die with zero sides",
            ),
            (
                GameError::PoolCannotBeEmpty,
                "refilling pool must have items with which to refill",
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
