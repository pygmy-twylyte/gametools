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
