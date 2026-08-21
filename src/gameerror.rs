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
    /// A card collection operation failed.
    #[error("card error: {0}")]
    CardError(#[from] CardError),
    /// A domino operation failed.
    #[error("domino error: {0}")]
    DominoError(#[from] DominoError),
    /// A dice construction or rolling operation failed.
    #[error("dice error: {0}")]
    DiceError(#[from] DiceError),
    /// A grid construction or access operation failed.
    #[error("grid error: {0}")]
    GridError(#[from] GridError),
    /// A refilling-pool operation failed.
    #[error("refilling pool error: {0}")]
    RefillingPoolError(#[from] RefillingPoolError),
    /// A spinner operation failed.
    #[error("spinner error: {0}")]
    SpinnerError(#[from] SpinnerError),
    /// A bounded-value operation failed.
    #[error("value error: {0}")]
    ValueError(#[from] ValueError),
    /// A pathfinding operation failed.
    #[error("pathfinding error: {0}")]
    PathfindingError(#[from] PathfindingError),
}

/// Errors specific to the grid module.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum GridError {
    /// One or both dimensions were zero.
    #[error("both grid dimensions must be > 0, got ({0},{1})")]
    InvalidSize(usize, usize),
    /// The requested grid area does not fit in `usize`.
    #[error("grid area overflows maximum integer value")]
    AreaOverflow,
    /// The supplied cell count differed from the grid's required area.
    #[error("actual cell count {actual} did not match expected size {expected}")]
    CellCountMismatch {
        /// Number of supplied cells.
        actual: usize,
        /// Number of cells required by the dimensions.
        expected: usize,
    },
    /// A dimension could not be represented by the grid's `i32` coordinates.
    #[error("very large {0} dimension truncated on conversion to i32::MAX")]
    DimensionTruncated(&'static str),
}

/// Errors specific to card collections and card transfer helpers.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum CardError {
    /// A draw was requested from an empty named collection.
    #[error("cannot draw from empty stack '{0}'")]
    StackEmpty(String),
    /// A collection did not contain enough cards for a multi-card draw.
    #[error("too few cards remain in '{0}' to satisfy need")]
    StackTooSmall(String),
    /// A requested card was absent from the source collection.
    #[error("the card sought was not found in this collection")]
    CardNotFound,
}

/// Errors specific to domino hands, trains, and bone piles.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum DominoError {
    /// The bone pile could not satisfy a draw.
    #[error("insufficient tiles left in the bone pile")]
    InsufficientTiles,
    /// A tile did not connect to the current train tail.
    #[error("that tile does not match the tail of the train")]
    TileUnconnected,
    /// A tile identifier was not present in the hand.
    #[error("tile with id '{0}' not found in hand")]
    TileNotFound(usize),
    /// A play targeted a closed train.
    #[error("attempted to play on a closed train")]
    TrainClosed,
}

/// Errors specific to spinners.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum SpinnerError {
    /// The spinner was empty or selected a covered wedge.
    #[error("spin() returned None: empty spinner or landed on covered wedge")]
    SpinnerEmpty,
}

/// Errors specific to [`crate::RefillingPool`].
#[derive(Debug, Clone, Error, PartialEq)]
pub enum RefillingPoolError {
    /// Pool construction was attempted with no items.
    #[error("refilling pool must have items with which to refill")]
    PoolCannotBeEmpty,
    /// An index did not identify an item in the pool.
    #[error("invalid index {0} for pool size {1}")]
    InvalidPoolIndex(usize, usize),
}

/// Errors specific to creating and rolling dice.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum DiceError {
    /// A die was created with zero sides.
    #[error("a die with zero sides cannot be created")]
    DieWithNoSides,
    /// An exploding-die trigger was not a valid side value.
    #[error("invalid explode trigger: {explode_on} on {sides}-sided die")]
    InvalidExplodeTrigger {
        /// Requested trigger value.
        explode_on: u64,
        /// Number of sides on the die.
        sides: u64,
    },
    /// A one-sided die would explode forever.
    #[error("one sided die would infinitely explode")]
    InfiniteExplosion,
}

/// Errors produced while validating weighted A* configuration.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum PathfindingError {
    /// A dynamic pxWD weight was outside `1.0..=2.0`.
    #[error("A* dynamic weight must be 1.0-2.0, got {0:.1}")]
    InvalidDynamicWeight(f32),
    /// A static weight was outside `0.0..=2.0`.
    #[error("A* static weight must be 0.0-2.0, got {0:.1}")]
    InvalidStaticWeight(f32),
}

/// Errors deriving from invalid values.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValueError {
    /// A lower bound exceeded its upper bound.
    #[error("min value must be less than max value")]
    MinOverMax,
    /// A value was outside its configured bounds.
    #[error("value outside valid range")]
    OutOfRange,
}

#[cfg(test)]
mod tests {
    use super::{
        CardError, DiceError, DominoError, GameError, GridError, PathfindingError,
        RefillingPoolError, SpinnerError, ValueError,
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
                GridError::InvalidSize(0, 2).into(),
                "grid error: both grid dimensions must be > 0, got (0,2)",
            ),
            (
                ValueError::OutOfRange.into(),
                "value error: value outside valid range",
            ),
            (
                PathfindingError::InvalidDynamicWeight(0.0).into(),
                "pathfinding error: A* dynamic weight must be 1.0-2.0, got 0.0",
            ),
            (
                PathfindingError::InvalidStaticWeight(8.0).into(),
                "pathfinding error: A* static weight must be 0.0-2.0, got 8.0",
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
