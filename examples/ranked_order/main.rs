//! `RankedOrder` example.
//!
//! Here we're getting ready for a turn, determining turn order for a large party.
//!
//! The `npc_generator` module uses a bunch of `RefillingPool`s to create a party with
//! randomized names and stats.
//!
//! The `initiative` closure is used to rank the characters for the turn using
//! a `DescendingOrder` (which is a `RankedOrder<_,_,Descending>`). They then
//! "take their turns" by popping from the order and calling their Npc::take_turn() method.
use gametools::Die;
use gametools::GameResult;
use gametools::ordering::DescendingOrder;

mod npc_generator;
use npc_generator::{Npc, NpcGenerator};

const PARTY_SIZE: usize = 30;

fn main() -> GameResult<()> {
    // Create a party of 30 characters.
    let big_party = create_characters(PARTY_SIZE)?;

    // Create a formula for determining turn order.
    let d20 = Die::new(20)?;
    let initiative =
        |npc: &Npc| (npc.dexterity + npc.speed / 5 + (d20.roll()) as u32) * 100 / npc.encumbrance;

    // Create a descending-ordered list of characters based on their initiative.
    let mut initiative_order = DescendingOrder::new();
    for character in big_party {
        initiative_order.push_with_ranker(character, initiative);
    }

    // "Take turns" by popping from the initiative order and calling Npc::take_turn().
    println!("Init | Npc::take_turn()");
    println!("----------------------------------");
    while let Some((npc, initiative)) = initiative_order.pop() {
        print!("{initiative:>3}: ");
        npc.take_turn();
    }
    Ok(())
}

fn create_characters(count: usize) -> GameResult<Vec<Npc>> {
    let mut npcgen = NpcGenerator::new()?;
    Ok((0..count).map(|_| npcgen.generate()).collect())
}
