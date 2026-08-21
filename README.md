[![Crates.io](https://img.shields.io/crates/v/gametools.svg)](https://crates.io/crates/gametools)
[![Docs.rs](https://docs.rs/gametools/badge.svg)](https://docs.rs/gametools)
[![Build Status](https://github.com/pygmy-twylyte/gametools/actions/workflows/test.yml/badge.svg)](https://github.com/pygmy-twylyte/gametools/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# gametools

**gametools** is a lightweight Rust library implementing components and mechanics common to tabletop and many other games. It's intended to be reusable and simplify the creation of games and game engines without blurring into the realm of physical simulation of apparatus or game-specific logic.

## Features

- `dice`: `Die` and `Rolls` support plain and exploding dice plus common roll analysis helpers like `histogram`, `highest`, `lowest`, and `count_where`.
- `cards`: extensible card/deck/hand/pile toolkit for custom face types, plus ready-made standard 52-card and Uno helpers.
- `dominos`: domino set creation, trains, hands, and longest-train solving.
- `grid`: point-addressed rectangular grids with bounded or toroidal neighbor traversal, row, and column helpers for boards, maps, pathfinding, and line-of-sight style algorithms.
- `fov`: field-of-view maps over `Grid<T>` using perimeter raycasting, recursive shadowcasting, and a reusable rectangle-based blocker cache.
- `pathfinding`: Dijkstra search maps plus A* and weighted A* paths with cardinal, diagonal, jump, custom, bounded, or toroidal movement.
- `metered_resource`: bounded unsigned counters for resources such as health, mana, stamina, or ammunition.
- `spinners`: weighted wedges with optional covering/blocking and chainable updates.
- `refilling_pool`: a randomized pool of any clonable type that refills itself when empty, with conditional and contextual draw helpers.
- `ordering`: `RankedOrder` and `PriorityQueue` for stable ranked lists or heap-backed priority scheduling, with min/max or ascending/descending aliases.
- Unit tests, doctests, and runnable examples across the crate.

## Example: Cards

```rust
use gametools::{AddCard, Card, CardCollection, CardFaces, Deck, Hand, TakeCard};

#[derive(Clone)]
struct Rune(char);

impl CardFaces for Rune {
    fn display_front(&self) -> String { format!("Rune {}", self.0) }
    fn display_back(&self) -> Option<String> { Some(String::from("Stone Tablet")) }
    fn matches(&self, other: &Self) -> bool { self.0 == other.0 }
    fn compare(&self, other: &Self) -> std::cmp::Ordering { self.0.cmp(&other.0) }
}

let runes = "FUTHARK".chars()
    .map(|glyph| Card::new_card(Rune(glyph)))
    .collect::<Vec<_>>();

let mut deck = Deck::from_cards("runes", runes);
deck.shuffle();

let mut hand = Hand::<Rune>::new("sage");
hand.add_cards(deck.take_cards(3));
assert_eq!(hand.size(), 3);
```

## Example: Grid

```rust
use gametools::{GameResult, Grid, GridSize, GridTopology, Point, PointDelta};

fn main() -> GameResult<()> {
    let size = GridSize::new(3, 3)?;
    let mut board = Grid::new_with_fn(size, |point| point.row * 10 + point.col)?;

    for (_, cell) in board.cardinal_neighbors_mut(Point::new(1, 1)) {
        *cell = 99;
    }

    assert_eq!(board[Point::new(1, 1)], 11);
    assert_eq!(board[Point::new(1, 0)], 99);

    let wrapped_west = board
        .step(Point::new(0, 0), PointDelta::WEST, GridTopology::Toroidal)
        .expect("a toroidal grid always resolves a step");
    assert_eq!(wrapped_west, Point::new(2, 0));
    Ok(())
}
```

## Example: Dice

```rust
use gametools::Die;

let rolls = Die::new(6).expect("d6 should be valid").roll_n(5);
let histogram = rolls.histogram();

if histogram.len() == 2 && histogram.values().any(|count| *count == 3) {
    println!("Full House!");
}
if histogram.len() == 1 {
    println!("Yahtzee!");
}
```

## Example: Spinners

```rust
use gametools::spinners::{Spinner, wedges_from_values};

let wedges = wedges_from_values(vec!["Rock", "Paper", "Scissors"]);
let spinner = Spinner::new(wedges);

if let Some(result) = spinner.spin() {
    println!("You chose: {result}");
}
```

## The Idea

This crate avoids hardcoding game rules. Instead, it provides flexible, composable abstractions to make building games easier, whether you're making a tabletop simulator, card game engine, board/map tool, or randomizer.

## Documentation

Full API docs with usage examples are available via [docs.rs](https://docs.rs/gametools).

## More Examples

See additional usage examples in the module docs:

- [Cards module](https://docs.rs/gametools/latest/gametools/cards/index.html): custom faces, deck/hand/pile traits, shuffling, drawing
- [Dice module](https://docs.rs/gametools/latest/gametools/dice/index.html): regular and exploding dice plus `Rolls` helpers
- [Dominos module](https://docs.rs/gametools/latest/gametools/dominos/index.html): longest-train solver
- [Grid module](https://docs.rs/gametools/latest/gametools/grid/index.html): point-addressed grids, neighbors, and row/column traversal
- [FOV module](https://docs.rs/gametools/latest/gametools/fov/index.html): perimeter raycasting, recursive shadowcasting, and reusable rectangle caches
- [Pathfinding module](https://docs.rs/gametools/latest/gametools/pathfinding/index.html): Dijkstra maps and A* search over bounded or toroidal grids
- [MeteredResource module](https://docs.rs/gametools/latest/gametools/metered_resource/index.html): bounded resources with saturating increase and reduction helpers
- [Ordering module](https://docs.rs/gametools/latest/gametools/ordering/index.html): ranked lists and priority queues
- [RefillingPool module](https://docs.rs/gametools/latest/gametools/refilling_pool/index.html): self-refilling random pools with contextual draws
- [Spinners module](https://docs.rs/gametools/latest/gametools/spinners/index.html): weighted wedges with optional blocking
- `cargo run --example cards`: ties the standard playing cards and Uno helpers together for a mini showdown
- `cargo run --example dice`: basic roll analysis, exploding dice, and poker-style histogram checks
- `cargo run --example grid`: point-addressed board traversal and chess-like attack maps
- `cargo run --example fov --features fov-egui`: interactive egui field-of-view visualization
- `cargo run --example pathfinding`: terminal Dijkstra and weighted-A* terrain-path demo
- `cargo run --example metered_resource`: bounded depletion, refill, and fraction-full behavior
- `cargo run --example refilling_pool`: an "infinite chest" that prefers loot based on character context
- `cargo run --example priority_queue`: ship attack ordering with `MinPriorityQ`
- `cargo run --example ranked_order`: initiative ordering with `DescendingOrder`

## License

Licensed under MIT. Contributions welcome!
