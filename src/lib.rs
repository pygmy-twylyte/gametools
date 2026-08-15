//! # gametools
//!
//! `gametools` provides reusable utilities for common game-building needs such as card decks,
//! dice, spinners, dominos, grids, ranked ordering, and bounded resources. The goal is to
//! provide flexible, modular tools to simplify prototyping and building games and simulations.
//!
//! ## Features
//! - `cards`: generic card faces plus deck, hand, and pile abstractions, with standard 52-card and Uno helpers.
//! - `dice`: `Die` and `Rolls` support for regular and exploding dice along with common roll-analysis helpers.
//! - `grid`: point-addressed rectangular grids with row, column, and neighbor traversal helpers.
//! - `fov`: field-of-view maps with raycasting, shadowcasting, and rectangle-based algorithms.
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
    CardError, DiceError, DominoError, GameError, GridError, RefillingPoolError, SpinnerError,
    ValueError,
};

pub mod grid;
pub use grid::{Grid, GridSize, GridTopology, Point, PointDelta};

pub mod fov;
pub use fov::{
    BlockingRect, FovMap, RectangleFov, perimeter_raycasting, perimeter_raycasting_into,
    rectangle_based_fov, rectangle_based_fov_into, recursive_shadowcasting,
    recursive_shadowcasting_into,
};

pub mod ordering;
pub use ordering::{
    AscendingOrder, DescendingOrder, Max, MaxPriorityQ, Min, MinPriorityQ, PriorityQueue,
    RankedOrder,
};

pub mod pathfinding;
pub use pathfinding::{
    MoveSet, Path, SearchMap, a_star, a_star_weighted, a_star_weighted_with_topology,
    a_star_with_topology, dijkstra_map, dijkstra_map_with_topology,
};

pub type GameResult<T> = Result<T, GameError>;

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
}
