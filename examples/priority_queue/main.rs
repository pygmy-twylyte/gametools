//! # `PriorityQueue` example.
//!
//! Here we have a small group of ships, each as some distance from a target, and each has known speed.
//! We'll base the attack order on how long it takes each unit to reach its target.
use gametools::ordering::MinPriorityQ;

fn main() {
    // Since we want to prioritize units with the *lowest* time to reach their target, we want
    // a `PriorityQueue<_,_,Min>` - but using the type alias `MinPriorityQ` is a little nicer.
    let mut attack_order = MinPriorityQ::new();

    // Build our fleet and push the units onto the heap, using time to reach target
    // as the priority.
    println!("Building the fleet...");
    let attackers = build_fleet();
    println!("Organizing the fleet...");
    for unit in attackers {
        let time_to_target = unit.distance / unit.speed;
        attack_order.push(unit, time_to_target);
    }

    // Since we used a MinPriorityQ, (= PriorityQueue<_,_,Min>) we get the unit with the
    // lowest time-to-target (ttt) in the pool with each pop(). We could add more
    // attackers at any time mid-stream and always get the nearest (time-wise) from pop().
    println!("Starting attack!");
    while let Some((attacker, ttt)) = attack_order.pop() {
        println!("    → {:<10} ({} min to target)", attacker.name, ttt);
    }
}

// Build a small fleet of naval units with a set speed and distance to some target.
fn build_fleet<'a>() -> Vec<NavalUnit<'a>> {
    vec![
        NavalUnit::from(("Fast Boat", 40, 10)),
        NavalUnit::from(("Cruiser", 30, 5)),
        NavalUnit::from(("Tugboat", 10, 2)),
        NavalUnit::from(("Submarine", 48, 8)),
    ]
}

#[derive(Debug, Clone)]
struct NavalUnit<'a> {
    name: &'a str,
    distance: u64,
    speed: u64,
}

impl<'a> From<(&'a str, u64, u64)> for NavalUnit<'a> {
    fn from((name, distance, speed): (&'a str, u64, u64)) -> Self {
        Self {
            name,
            distance,
            speed,
        }
    }
}
