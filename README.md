[![Crates.io](https://img.shields.io/crates/v/gametools.svg)](https://crates.io/crates/gametools)
[![Docs.rs](https://docs.rs/gametools/badge.svg)](https://docs.rs/gametools)
[![Build Status](https://github.com/pygmy-twylyte/gametools/actions/workflows/test.yml/badge.svg)](https://github.com/pygmy-twylyte/gametools/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

# gametools

**gametools** is a lightweight Rust library for simulating game components like dice rolls, extensible card decks, dominos, and spinners. It's designed to be modular, testable, and usable in both game engines and CLI tools. The goal is to provide reusable game apparatus — not game logic — so you can build your rules on top of well-tested building blocks.

## Features

- 🎲 Numeric dice up to 255 sides, plus dice pools with chainable operations
- 🃏 Extensible cards toolkit: compose custom face types, deck/hand/pile flows, plus ready-made standard 52-card and Uno helpers
- 🁫 Dominos with support for longest-path train solving
- 🌀 Spinners with support for weighted wedges and optional blocking
- 💥 Human-readable game errors for common failure conditions
- 🧪 Well-documented and tested with 90%+ code coverage

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

let mut deck = Deck::new("runes", runes);
deck.shuffle();

let mut hand = Hand::<Rune>::new("sage");
hand.add_cards(deck.take_cards(3));
assert_eq!(hand.size(), 3);
```

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

## The Idea

This crate avoids hardcoding game rules. Instead, it provides flexible, composable abstractions to make building games easier — whether you're making a tabletop simulator, card game engine, or randomizer tool.

## Documentation

Full API docs with usage examples are available via [docs.rs](https://docs.rs/gametools).

## More Examples

See additional usage examples in the module docs:

- [Cards module](https://docs.rs/gametools/latest/gametools/cards/index.html): custom faces, deck/hand/pile traits, shuffling, drawing
- [Dominos module](https://docs.rs/gametools/latest/gametools/dominos/index.html): longest-train solver
- `examples/cards`: a mashup that ties the standard playing cards and Uno modules together for a mini showdown

The examples/yahtzee folder contains an example of a Yahtzee-playing agent created using the Dice module.

## License

Licensed under MIT. Contributions welcome!
