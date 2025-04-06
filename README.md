# gametools

**gametools** is a lightweight Rust library for handling game mechanics like dice rolls, card decks, and dominos. It's designed to be modular, testable, and usable in both game engines and CLI tools. The 
goal is to provide the core functionality of the apparatus, making it easier to implement game logic.

## Features

- Numeric dice of up to 255 sides, and pools of dice
- Chainable dice pool operations
- Support for cards and dominos
- DominoHand includes pathfinding algorithm to find longest plays
- Useful game errors for common failure conditions

## Example

```rust
use gametools::{Die, DicePool};

let d6 = Die::new(6);
let total = d6.roll_into_pool(5)
    .take_highest(4)
    .nerf(1)
    .sum();
```

Many more examples are available in doctests and unit tests. 
