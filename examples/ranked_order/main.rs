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
    fn initiative(&self, die: &Die) -> u64 {
        // expressing encumbrance as an integer % of some max (60 = 60%, 110 = 110%) allows us to
        // avoid casting by using a u64. Multiplying by 100/enc = dividing by enc/100.
        (self.dex / 5 + die.roll() + self.modifier.unwrap_or_default()) * 100 / self.encumbrance
    }
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
    let militia = create_characters(30)?;
    let mut turn_order = DescendingOrder::new();
    for character in militia {
        let initiative = character.initiative(&d20);
        turn_order.push(character, initiative);
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
