use gametools::GameResult;
use gametools::spinners::{Spinner, Wedge};

fn main() -> GameResult<()> {
    /*

    This space is currently used for working up examples and testing.

     */
    let wedges = vec![
        Wedge::new("Prepare".to_string(), 3, true),
        Wedge::new("Sit Out".to_string(), 2, true),
        Wedge::new("Clean Up".to_string(), 1, true),
    ];

    let spinner = Spinner::new(wedges);
    for _ in 1..18 {
        println!("{}", spinner.spin().unwrap_or("None".to_string()));
    }
    Ok(())
}
