use gametools::{GameResult, RefillingPool};

static FIRST_NAMES: [&str; 18] = [
    "Dave", "Chris", "Kathleen", "Andrew", "Emily", "Echo", "Pullo", "Finn", "Mary", "Peggy",
    "Rorie", "Leland", "Penelope", "Hari", "Gaal", "Hober", "Poly", "Constant",
];

static LAST_NAMES: [&str; 9] = [
    "Stinkfoot",
    "Garglebottom",
    "Grabbutt",
    "Whoomi",
    "Shrubber",
    "Dornick",
    "Seldon",
    "Mallow",
    "Verisof",
];
pub struct Npc {
    pub(crate) name: String,
    pub(crate) strength: u32,
    pub(crate) dexterity: u32,
    pub(crate) stamina: u32,
    pub(crate) speed: u32,
    pub(crate) encumbrance: u32,
}
impl Npc {
    pub fn take_turn(&self) {
        println!("→ {} takes a turn…", self.name)
    }
}

pub struct NpcGenerator {
    first_names: RefillingPool<String>,
    last_names: RefillingPool<String>,
    strengths: RefillingPool<u32>,
    dexterities: RefillingPool<u32>,
    staminas: RefillingPool<u32>,
    speeds: RefillingPool<u32>,
    encumbrances: RefillingPool<u32>,
}
impl NpcGenerator {
    pub fn new() -> GameResult<Self> {
        Ok(Self {
            first_names: RefillingPool::new(FIRST_NAMES.map(|s| s.to_string()))?,
            last_names: RefillingPool::new(LAST_NAMES.map(|s| s.to_string()))?,
            strengths: RefillingPool::new(8..19)?,
            dexterities: RefillingPool::new(8..19)?,
            staminas: RefillingPool::new(40..70)?,
            speeds: RefillingPool::new(1..5)?,
            encumbrances: RefillingPool::new(80..110)?,
        })
    }

    pub fn generate(&mut self) -> Npc {
        Npc {
            name: format!("{} {}", self.first_names.draw(), self.last_names.draw()),
            strength: self.strengths.draw(),
            dexterity: self.dexterities.draw(),
            stamina: self.staminas.draw(),
            speed: self.speeds.draw(),
            encumbrance: self.encumbrances.draw(),
        }
    }
}
