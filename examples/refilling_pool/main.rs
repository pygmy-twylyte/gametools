//! `RefillingPool` as "Infinite Chest" example.
//!
//! Three `RefillingPool`s are used for this example:
//! - `infinite_chest` is filled with 12 loot items; it refills itself when empty
//! - `character_kinds` is a pool of character classes used to generate test contexts
//! - `hp_configs` is a pool of HP values, also used to generate test character/contexts
//!
//! `loot_matcher` is a closure that evaluates an item in a given character/HP context and
//! determines whether that item would be preferred loot for that character.
use context::{CharacterKind, Context};
use loot::{Loot, LootKind};

use gametools::GameResult;
use gametools::refilling_pool::RefillingPool;

fn main() -> GameResult<()> {
    use LootKind::*;

    let loot_items = [
        (Weapon, "Longsword"),
        (Weapon, "Shortsword +1"),
        (Weapon, "Halberd"),
        (HealthPotion, "Blue Health +10"),
        (HealthPotion, "Violet Health +20"),
        (MagicItem, "Amulet of Protection +1"),
        (MagicItem, "Boots of Speed +1"),
        (ManaPotion, "Wizard's Flask"),
        (ManaPotion, "Alchemist's Ale"),
        (SpellBook, "Magic Missile"),
        (SpellBook, "Conjure Familiar"),
        (SpellBook, "Levitation"),
    ]
    .into_iter()
    .map(Loot::from)
    .collect::<Vec<_>>();

    // We can draw loot from this chest forever.
    let mut infinite_chest = RefillingPool::new(loot_items)?;

    // This closure is used to select any preferred items in the chest, given the context.
    let loot_matcher = |context: &Context, loot: &Loot| match loot.kind {
        HealthPotion => context.current_hp < 10,
        MagicItem => true,
        ManaPotion => context.class == CharacterKind::Wizard,
        SpellBook => context.class == CharacterKind::Wizard,
        Weapon => context.class == CharacterKind::Fighter,
    };

    // Here we create a couple of small pools to be used in random character (context) generation...
    let mut character_kinds = RefillingPool::new([CharacterKind::Fighter, CharacterKind::Wizard])?;
    let mut hp_configs = RefillingPool::new([5, 8, 12, 20])?;

    // Test run -- generate a bunch characters and have each draw an item from the infinite chest.
    // Each should get mostly appropriate loot, with a few stinkers when the unused pool is near
    // empty and none of the preferred items remain (until the next refill).
    let character_count = 100;
    println!(
        "{character_count} characters drawing from the infinite chest with refill pool of {} items...",
        infinite_chest.full_size()
    );
    for _ in 0..character_count {
        let context = Context::new(character_kinds.draw(), hp_configs.draw());
        let loot = infinite_chest.draw_with_context_or_any(&context, loot_matcher);
        println!("Character [{context}] → {loot}");
    }

    Ok(())
}

/// A very basic module to demonstrate how an arbitrary type can be used or created for
/// the contextual draw methods in `RefillingPool`.
mod context {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(super) struct Context {
        pub(super) class: CharacterKind,
        pub(super) current_hp: u64,
    }

    impl Context {
        pub(super) fn new(character: CharacterKind, current_hp: u64) -> Self {
            Self {
                class: character,
                current_hp,
            }
        }
    }

    impl std::fmt::Display for Context {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{} ({} hp)",
                match self.class {
                    CharacterKind::Fighter => "Fighter",
                    CharacterKind::Wizard => "Wizard",
                },
                self.current_hp
            )
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(super) enum CharacterKind {
        Fighter,
        Wizard,
    }
}

mod loot {
    #[derive(Debug, Clone, PartialEq)]
    pub(super) struct Loot {
        pub(super) name: String,
        pub(super) kind: LootKind,
    }
    impl From<(LootKind, &str)> for Loot {
        fn from(value: (LootKind, &str)) -> Self {
            Self {
                kind: value.0,
                name: value.1.to_string(),
            }
        }
    }
    impl std::fmt::Display for Loot {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} ({:?})", self.name, self.kind)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(super) enum LootKind {
        Weapon,
        HealthPotion,
        MagicItem,
        ManaPotion,
        SpellBook,
    }
}
