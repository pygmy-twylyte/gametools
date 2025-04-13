use gametools::GameResult;
use gametools::GameError;
use gametools::spinners::{Spinner, Wedge};

fn main() -> GameResult<()> {
    /*

    This space is currently used for working up examples and testing.

     */
    let numeric_spinner = Spinner::new(vec![
        Wedge::new(5),
        Wedge::new(10),
        Wedge::new(15),
        Wedge::new(20),
    ]);
    let spin = numeric_spinner.spin().ok_or( GameError::SpinnerEmpty)?;
    println!("You win ${spin}!");
    
    Ok(())
}
