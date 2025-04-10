# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
