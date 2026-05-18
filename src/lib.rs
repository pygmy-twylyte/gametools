//! # gametools
//!
//! `gametools` provides utilities for working with common table game apparatus such as card decks,
//! dice, spinners, and dominos. The goal is to provide flexible, modular tools to simplify prototyping and building
//! games and simulations.
//!
//! ## Features
//! - `cards`: generic card faces plus deck, hand, and pile abstractions, with standard 52-card and Uno helpers.
//! - `dice`: `Die` and `Rolls` support for regular and exploding dice along with common roll-analysis helpers.
//! - `ordering`: stable ranked lists (`RankedOrder`) and heap-backed queues (`PriorityQueue`) for turn order and scheduling.
//! - `metered_resource`: bounded unsigned counters with saturating increase and reduction helpers.
//! - `refilling_pool`: infinitely reusable random pools with conditional and contextual draw helpers.
//! - `spinners`: decision wheels with weighted, coverable wedges that can hold arbitrary values.
//! - `dominos`: domino set creation, train management, and longest-train solving.
//! - Module-specific error enums plus `GameError` / `GameResult` for aggregate error handling across the crate.

pub mod cards;
pub use cards::{
    AddCard, Card, CardCollection, CardFaces, CardHand, Deck, Hand, Pile, Rank, Suit, TakeCard,
};

pub mod dice;
pub use dice::{Die, DieResult, Rolls};

pub mod dominos;
pub use dominos::{BonePile, Domino, DominoHand, MAX_PIPS, Train};

pub mod metered_resource;
pub use metered_resource::MeteredResource;

pub mod refilling_pool;
pub use refilling_pool::RefillingPool;

pub mod spinners;
pub use spinners::{Spinner, Wedge, wedges_from_tuples, wedges_from_values};

pub mod gameerror;
pub use gameerror::{
    CardError, DiceError, DominoError, GameError, RefillingPoolError, SpinnerError, ValueError,
};
pub type GameResult<T> = Result<T, GameError>;

pub mod ordering;
pub use ordering::{
    AscendingOrder, DescendingOrder, Max, MaxPriorityQ, Min, MinPriorityQ, PriorityQueue,
    RankedOrder,
};
