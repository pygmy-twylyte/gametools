[![Crates.io](https://img.shields.io/crates/v/gametools.svg)](https://crates.io/crates/gametools)
[![Docs.rs](https://docs.rs/gametools/badge.svg)](https://docs.rs/gametools)
[![Build Status](https://github.com/pygmy-twylyte/gametools/actions/workflows/test.yml/badge.svg)](https://github.com/pygmy-twylyte/gametools/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# gametools

**gametools** is a lightweight Rust library for simulating game components like dice rolls, card decks, dominos, and spinners. It's designed to be modular, testable, and usable in both game engines and CLI tools. The goal is to provide reusable game apparatus — not game logic — so you can build your rules on top of well-tested building blocks.

## Features

- 🎲 Numeric dice up to 255 sides, plus dice pools with chainable operations
- 🃏 Standard playing cards: decks, hands, piles, shuffling, drawing
- 🁫 Dominos with support for longest-path train solving
- 🌀 Spinners with support for weighted wedges and optional blocking
- 💥 Human-readable game errors for common failure conditions
- 🧪 Well-documented and tested with 90%+ code coverage

## Example: Dice

```rust
use gametools::{Die, DicePool};

let d6 = Die::new(6);
let total = d6.roll_into_pool(5)
    .take_highest(4)
    .nerf(1)
    .sum();
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

## Philosophy

This crate avoids hardcoding game rules. Instead, it provides flexible, composable abstractions to make building games easier — whether you're making a tabletop simulator, card game engine, or randomizer tool.

## Documentation

Full API docs with usage examples are available via [docs.rs](https://docs.rs/gametools).

## More Examples

See additional usage examples in the module docs:

- [Cards module](https://docs.rs/gametools/latest/gametools/cards/index.html): shuffling, drawing, hands
- [Dominos module](https://docs.rs/gametools/latest/gametools/dominos/index.html): longest-train solver

## License

Licensed under MIT. Contributions welcome!
