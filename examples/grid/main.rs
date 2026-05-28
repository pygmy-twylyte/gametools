use std::collections::HashMap;
use std::fmt::Display;

use gametools::GameResult;
use gametools::{Grid, GridError, GridSize};

const HIT: char = '¤';
const MISS: char = '·';

fn main() -> GameResult<()> {
    println!("For demonstration, we use the lowercase alphabet to populate a");
    println!("Grid<char>:");
    let dims = GridSize::new(8, 3)?;
    let chars: Vec<char> = ('a'..='z').take(dims.area()?).collect();
    let grid = Grid::from_vec(dims, chars)?;
    show_grid(&grid, "Alphabet with GridSize(8, 3)");

    println!("Here's another built from the same chars, but GridSize(6,4):");
    let dims = GridSize::new(
        6, 4, /* try changing these and see how output is affected */
    )?;
    // error if there are more cells in the grid than there are letters
    if dims.area()? > 26 {
        return Err(GridError::AreaOverflow.into());
    }
    let chars: Vec<char> = ('a'..='z').take(dims.area()?).collect();
    let grid = Grid::from_vec(dims, chars)?;
    show_grid(&grid, "Alphabet with GridSize(6, 4)");
    println!(
        "That {} x {} configuration is used for the examples below:",
        grid.size().width(),
        grid.size().height()
    );

    // make a map of the letters to their points on the grid in this
    // configuration, for easy reference
    let point_of = grid
        .iter()
        .fold(HashMap::new(), |mut acc, (point, &value)| {
            acc.insert(value, point);
            acc
        });

    // let's say 'r' is for Rook.
    let r_point = point_of[&'r'];
    // here we create the grid with a default value first
    let mut rook_attack_grid = Grid::new(dims, MISS)?;
    // and then mutate the values at points in the same row or column
    for (_, ltr) in rook_attack_grid.col_at_mut(r_point) {
        *ltr = HIT;
    }
    for (_, ltr) in rook_attack_grid.row_at_mut(r_point) {
        *ltr = HIT;
    }
    rook_attack_grid[r_point] = 'R';
    show_grid(&rook_attack_grid, "Attack lines, treating R as a Rook");

    // ...and 'b' is for Bishop
    let b_point = point_of[&'b'];
    // here, we build the grid in one step using an Fn(Point) -> T closure
    let bishop_attack_grid = Grid::new_with_fn(dims, |p| {
        if p == b_point {
            'B'
        } else if p.same_diagonal(b_point) {
            HIT
        } else {
            MISS
        }
    })?;
    show_grid(&bishop_attack_grid, "Attack lines, treating B as a Bishop");

    // ... and 'q' is for Queen ...
    let q_point = point_of[&'q'];
    let queen_attack_grid = Grid::new_with_fn(dims, |p| {
        if p == q_point {
            'Q'
        } else if p.same_diagonal(q_point) || p.same_col(q_point) || p.same_row(q_point) {
            HIT
        } else {
            MISS
        }
    })?;
    show_grid(&queen_attack_grid, "Attack lines, treating Q as a Queen");

    // ... and 'k' is for king ...
    let k_point = point_of[&'k'];
    let king_attack_grid = Grid::new_with_fn(dims, |p| {
        if p == k_point {
            'K'
        } else if grid.all_neighbors(p).any(|(pt, _)| pt == k_point) {
            HIT
        } else {
            MISS
        }
    })?;
    show_grid(&king_attack_grid, "Attack lines, treating K as a King");

    // ... how many turns would it take the King to reach any cell? ...
    let king_path = Grid::new_with_fn(dims, |p| (p - k_point).distance_chebyshev())?;
    show_grid(&king_path, "Turns for (K)ing to reach any cell");

    Ok(())
}

fn show_grid<T: Display>(grid: &Grid<T>, caption: impl AsRef<str>) {
    // print grid values in tabular format
    print!("↓ {} ↓", caption.as_ref());
    grid.iter().enumerate().for_each(|(idx, (_, value))| {
        if idx % grid.size().width() == 0 {
            println!();
        }
        print!("{:^3}", value);
    });
    println!("\n");
}
