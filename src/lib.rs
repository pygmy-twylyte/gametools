//! # gametools
//!
//! `gametools` provides utilities for working with common table game apparatus such as card decks,
//! dice, and dominos. The goal is to provide flexible, modular tools to simplify prototyping and building
//! games and simulations.
//!
//! ## Features
//! - Standard 52-card deck handling with tools for decks, piles, and hands.
//! - Numeric dice with up to 255 sides
//! - Tools for playing with pools of dice
//! - Domino set creation (up to full double-18) and management.
//! - Pathfinding with backtracking + pruning to find optimum domino train in a hand.
//! - Custom GameResult and GameError types to help with common game conditions.

pub mod dice;
pub use dice::{DicePool, Die};

pub mod cards;
pub use cards::{AddCard, Card, CardHand, Deck, Pile, Rank, Suit, TakeCard};

pub mod dominos;
pub use dominos::{BonePile, Domino, DominoHand, MAX_PIPS, Train};

pub mod spinners;

pub mod gameerror;
pub use gameerror::GameError;

pub type GameResult<T> = Result<T, GameError>;
