//! `RankedOrder` example.
//!
use gametools::Die;
use gametools::GameResult;
use gametools::RefillingPool;
use gametools::ordering::DescendingOrder;

struct RpgCharacter {
    name: String,
    dex: u64,
    encumbrance: u64,
    modifier: Option<u64>,
}
impl RpgCharacter {
    fn take_turn(&self) {
        println!(
            "{} (dex:{} mod:{} enc:{}) takes a turn → ",
            self.name,
            self.dex,
            self.modifier.unwrap_or_default(),
            self.encumbrance
        );
    }
}

fn main() -> GameResult<()> {
    let d20 = Die::new(20).unwrap();
    let initiative = |npc: &RpgCharacter| {
        (npc.dex / 5 + d20.roll() + npc.modifier.unwrap_or_default()) * 100 / npc.encumbrance
    };
    let militia = create_characters(30)?;
    let mut turn_order = DescendingOrder::new();
    for character in militia {
        turn_order.push_with_ranker(character, initiative);
    }
    while let Some((npc, _)) = turn_order.pop() {
        npc.take_turn();
    }
    Ok(())
}

fn create_characters(count: usize) -> GameResult<Vec<RpgCharacter>> {
    let mut first_names = RefillingPool::new([
        "Grog", "Dart", "Frood", "Stiv", "Binnt", "Urexa", "Risto", "Fanna", "Kukaa",
    ])?;
    let mut last_names = RefillingPool::new([
        "Sharder",
        "Leppre",
        "Broilan",
        "Measterton",
        "Shale",
        "Blantt",
        "Phluphem",
        "Queempai",
    ])?;
    let mut dex_values = RefillingPool::new(8..18)?;
    let mut encumbrances = RefillingPool::new(80..120)?;
    let mut modifiers = RefillingPool::new([Some(5), Some(3), None, None, None, None])?;

    Ok((0..count)
        .map(|_| RpgCharacter {
            name: format!("{} {}", first_names.draw(), last_names.draw()),
            dex: dex_values.draw(),
            encumbrance: encumbrances.draw(),
            modifier: modifiers.draw(),
        })
        .collect())
}
