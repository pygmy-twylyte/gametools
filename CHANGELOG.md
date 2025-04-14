# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
