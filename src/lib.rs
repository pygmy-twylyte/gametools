//! # gametools
//!
//! `gametools` provides utilities for working with common table game apparatus such as card decks,
//! and dice. The goal is to give you flexible, modular tools to simplify prototyping and building
//! tabletop or digital games.
//!
//! ## Features
//! - Standard 52-card deck handling
//! - Numeric dice with up to 255 sides
//! - Functions for working with pools of dice

mod dice;
pub use dice::{DicePool, Die};

mod cards;
pub use cards::{AddCard, Card, Deck, CardHand, Pile, Rank, Suit, TakeCard};

mod dominos;
pub use dominos::{Domino, BonePile, MAX_PIPS, Train, DominoHand};

mod gameerror;
pub use gameerror::GameError;

pub type GameResult<T> = Result<T, GameError>;
