//! Shared error types used across the crate.
//!
//! Module-specific errors stay narrow enough for callers to handle precisely,
//! while [`GameError`] aggregates them for APIs that can surface failures from
//! more than one subsystem.
//!

use thiserror::Error;

/// Aggregate error type for APIs that may surface failures from multiple modules.
#[derive(Debug, Error, PartialEq)]
pub enum GameError {
    #[error("card error: {0}")]
    CardError(#[from] CardError),
    #[error("domino error: {0}")]
    DominoError(#[from] DominoError),
    #[error("dice error: {0}")]
    DiceError(#[from] DiceError),
    #[error("refilling pool error: {0}")]
    RefillingPoolError(#[from] RefillingPoolError),
    #[error("spinner error: {0}")]
    SpinnerError(#[from] SpinnerError),
    #[error("value error: {0}")]
    ValueError(#[from] ValueError),
}

/// Errors specific to card collections and card transfer helpers.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum CardError {
    #[error("cannot draw from empty stack '{0}'")]
    StackEmpty(String),
    #[error("too few cards remain in '{0}' to satisfy need")]
    StackTooSmall(String),
    #[error("the card sought was not found in this collection")]
    CardNotFound,
}

/// Errors specific to domino hands, trains, and bone piles.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum DominoError {
    #[error("insufficient tiles left in the bone pile")]
    InsufficientTiles,
    #[error("that tile does not match the tail of the train")]
    TileUnconnected,
    #[error("tile with id '{0}' not found in hand")]
    TileNotFound(usize),
    #[error("attempted to play on a closed train")]
    TrainClosed,
}

/// Errors specific to spinners.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum SpinnerError {
    #[error("spin() returned None: empty spinner or landed on covered wedge")]
    SpinnerEmpty,
}

/// Errors specific to [`crate::RefillingPool`].
#[derive(Debug, Clone, Error, PartialEq)]
pub enum RefillingPoolError {
    #[error("refilling pool must have items with which to refill")]
    PoolCannotBeEmpty,
    #[error("invalid index {0} for pool size {1}")]
    InvalidPoolIndex(usize, usize),
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

/// Errors deriving from invalid values.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValueError {
    #[error("min value must be less than max value")]
    MinOverMax,
    #[error("value outside valid range")]
    OutOfRange,
}

#[cfg(test)]
mod tests {
    use super::{
        CardError, DiceError, DominoError, GameError, RefillingPoolError, SpinnerError, ValueError,
    };
    use std::error::Error;

    #[test]
    fn test_game_error_display_and_trait() {
        let cases: Vec<(GameError, &str)> = vec![
            (
                CardError::StackEmpty("Main".to_string()).into(),
                "card error: cannot draw from empty stack 'Main'",
            ),
            (
                CardError::StackTooSmall("Reserve".to_string()).into(),
                "card error: too few cards remain in 'Reserve' to satisfy need",
            ),
            (
                CardError::CardNotFound.into(),
                "card error: the card sought was not found in this collection",
            ),
            (
                DominoError::InsufficientTiles.into(),
                "domino error: insufficient tiles left in the bone pile",
            ),
            (
                DominoError::TileUnconnected.into(),
                "domino error: that tile does not match the tail of the train",
            ),
            (
                DominoError::TrainClosed.into(),
                "domino error: attempted to play on a closed train",
            ),
            (
                SpinnerError::SpinnerEmpty.into(),
                "spinner error: spin() returned None: empty spinner or landed on covered wedge",
            ),
            (
                RefillingPoolError::PoolCannotBeEmpty.into(),
                "refilling pool error: refilling pool must have items with which to refill",
            ),
            (
                DiceError::DieWithNoSides.into(),
                "dice error: a die with zero sides cannot be created",
            ),
            (
                ValueError::OutOfRange.into(),
                "value error: value outside valid range",
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
