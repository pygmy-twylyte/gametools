use gametools::{GameResult, metered_resource::MeteredResource};

fn main() -> GameResult<()> {
    println!("Create a health pool for something, max HP = 75 →");
    let mut health: MeteredResource<u32> = MeteredResource::new("hp", 0, 75, 75).unwrap();
    show(&health);

    println!("Character is hit for 22 hp damage.");
    health.reduce_by(22);
    show(&health);

    println!("The .fraction_left() method gives us the fraction of HP remaining.");
    println!("Fraction full: {:.2}", health.fraction_left());

    println!("Hit for 1000 hp →");
    health.reduce_by(1000);
    show(&health);

    println!("Healer intervenes... and also boosts max HP...");
    health.increase_by(75);
    health = health.with_new_bounds(0, 100)?;
    show(&health);
    Ok(())
}

fn show(res: &MeteredResource<u32>) {
    println!(
        "{} {} ({} - {})",
        res.current(),
        res.unit(),
        res.min(),
        res.max()
    )
}
