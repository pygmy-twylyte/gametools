# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
---
## [0.7.0] - 2025-11-14

### Changed
- `Wedge::cover`/`uncover` now operate on borrowed wedges and clone only what they need, avoiding the `Clone` + `mut` dance at each call site.
- Split the `Spinner<T>` impl blocks so general helpers (e.g., `spin`, `add_wedge`) only require `T: Clone`, and the comparison-driven helpers additionally bound `T: PartialEq`.
- `Spinner::cover`, `uncover`, `remove_wedges`, and `replace_value` now accept borrowed match values so callers can avoid cloning owned types; `replace_value` still consumes the replacement so it works for owned values as well.
- Documentation and unit tests updated to reflect the borrowed-parameter API, plus a regression test proving `replace_value` functions for `String` wedges.

---
## [0.6.0] - 2025-12-15

### Added
- `uno_cards` module covering creation of and some typical manipulations of standard Uno decks and hands
- `UnoCard::plays_on` centralizes Uno card eligibility checks, and `Hand<UnoCard>::playable_on` surfaces the playable indices for a given discard.
- Targeted unit tests for Uno matching scenarios, including declared colors and action cards.
- A refreshed `examples/cards` program that pairs a high-card showdown with an Uno color rally to highlight cross-module workflows.

### Changed
- README emphasizes Uno coverage within the cards toolkit and links directly to the new cards example.

---
## [0.5.0] - 2025-11-02

### Added
- `CardFaces` trait plus enriched `Card<T>` metadata (UUIDs, deck ownership) for building arbitrary card games.
- Shared `CardCollection`, `AddCard`, and `TakeCard` traits so helpers can work with decks, hands, and piles interchangeably.
- New deck helpers including `Deck::new_from_faces`, `Deck::deal`, and trait-powered `take_cards` flows for smoother game orchestration.

### Changed
- Replaced the playing cards module with a fully modular cards toolkit; standard 52-card support now lives under `cards::std_playing_cards`.
- `Deck`, `Hand`, and `Pile` now operate on generic `Card<T>` values, enabling custom face types across the whole crate.
- Refreshed documentation and examples to highlight the new cards architecture and usage patterns.

---
## [0.4.0] - 2025-07-18

### Added
- optional `serde` derivations for most library structs / enums ({..., features = "serde"} to enable)

---
## [0.3.2] -

### Added
- Die::try_new() and Die::try_roll_into_pool(). Non-panic versions of new() and roll_into_poll().
- GameErrors corresponding to failures of the above try_ functions.

---
## [0.3.1] - 2025-05-11

### Added
- YahtzAI -- an example that uses the dice module to create a Yahtzee AI which checks the expected outcome of all possible rerolls to optimize scoring
- DicePool::reroll_by_idx() to reroll some dice within the pool, specified by index rather than value (unlike DicePool::reroll_if())

---
## [0.3.0] - 2025-05-01

### Added
- DicePool::binned_rolls() to create a hash of roll value counts, aiding in scoring for various games.

### Changed
- DicePool::count_success_using() renamed to count_if() for consistency, brevity, and clarity.
- DicePool::count_success_over() renamed to count_over() for consistency, brevity, and clarity.

---
## [0.2.0] - 2025-04-13

### Added
- Introduced the new `spinners` module:
  - `Spinner<T>` supports weighted or uniform wedges, random selection, and optional wedge blocking.
  - `Wedge<T>` struct represents individual outcomes with customizable weight and active/inactive status.
  - Includes helper functions `wedges_from_values()` and `wedges_from_tuples()` for quick setup.
- Full unit test coverage for the spinners module.
- Added `README.md` examples for spinners and linked additional module docs.
- Added badges to `README.md` for crates.io, docs.rs, CI, and license.

### Changed
- Improved documentation across the crate, especially for `cards`, `dominos`, and `spinners`.
- Added doc comments for all public items in `spinners.rs`, including full usage examples.
- Minor wording improvements in other doc comments to ensure consistency.

---

## [0.1.1] - 2025-04-09

### Added
- Added `CardHand::contains_by_rs()` convenience method to search for a card by rank and suit without instantiating a `Card`.

### Changed
- Refactored `DicePool::range()` for simpler and clearer implementation.
- Added full unit test coverage for the `gameerror` module (previously untested).
- Completed 100% unit test coverage for all other modules.

---

## [0.1.0] - 2025-04-05

### Added
- Initial release of `gametools` crate.
- Core modules:
  - `cards`: Represent playing cards, decks, hands, and piles.
  - `dice`: Dice rolling utilities with multiple die types.
  - `dominos`: Domino representation, hands, trains, and train-solving logic.
  - `gameerror`: Unified error enum for common gameplay logic failures.
- `dominos` module includes a solver that computes the longest valid train from a hand.
- Internal test suite covering 60%+ of crate behavior.
- Rustdoc documentation for all public items.
