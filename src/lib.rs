//! # gametools
//!
//! `gametools` provides utilities for working with common table game apparatus such as card decks,
//! dice, spinners, and dominos. The goal is to provide flexible, modular tools to simplify prototyping and building
//! games and simulations.
//!
//! ## Features
//! - Cards module that supports cards of any type, with 1 or 2 faces.
//! - Pre-defined StandardCard type for standard playing cards.
//! - Hand analytics for standard cards (detect straight, "n" of a kind, etc.) including Joker / wildcard handling.
//! - Numeric dice with up to 255 sides.
//! - Tools for playing with and transforming pools of dice.
//! - Spinners (random selectors) with "wedges" returning arbitrary types and can be covered/blocked or weighted.
//! - Domino set creation (up to full double-18) and management.
//! - Pathfinding with backtracking + pruning to find optimum domino train in a hand.
//! - Custom GameResult and GameError types to help with common game conditions.

pub mod dice;
pub use dice::{DicePool, Die};

pub mod cards;
pub use cards::{
    AddCard, Card, CardCollection, CardFaces, CardHand, Deck, Hand, Pile, Rank, Suit, TakeCard,
};

pub mod dominos;
pub use dominos::{BonePile, Domino, DominoHand, MAX_PIPS, Train};

pub mod spinners;
pub use spinners::{Spinner, Wedge, wedges_from_tuples, wedges_from_values};

pub mod gameerror;
pub use gameerror::GameError;

pub type GameResult<T> = Result<T, GameError>;
