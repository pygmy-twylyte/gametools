use std::collections::HashMap;
use std::hash::Hash;

use gametools::{
    GameResult, Grid, GridSize, MoveSet, Path, Point, PointDelta, dijkstra_map,
    pathfinding::{Cost, HeuristicWeight, a_star_weighted, path_from_search_map},
};

use colored::Colorize;
use rand::seq::SliceRandom as _;

const START: Point = Point::new(25, 4);
const GOAL: Point = Point::new(3, 9);
const MOVE_SET: MoveSet = MoveSet::EightWay;
const CARDINAL_COST: Cost = 10;
const DIAGONAL_COST: Cost = 14;
const MEAN_COST: Cost = (CARDINAL_COST + DIAGONAL_COST) / 2;
const GOAL_DISTANCE_MIN: u32 = 40;
const TERRAIN: [(char, Option<Cost>); 4] =
    [('.', Some(1)), ('"', Some(2)), ('~', Some(4)), ('#', None)];

fn main() -> GameResult<()> {
    let map = read_map_into_grid("examples/pathfinding/test_map.txt")?;
    show_grid(&map, "Initial map");

    let mut pathable_points: Vec<_> = map
        .iter()
        .filter_map(|(p, v)| if *v != '#' { Some(p) } else { None })
        .collect();
    pathable_points.shuffle(&mut rand::rng());

    let goal = pathable_points.pop().unwrap_or(GOAL);
    let start = pathable_points
        .iter()
        .find(|&&p| (goal - p).distance_chebyshev() >= GOAL_DISTANCE_MIN)
        .copied()
        .unwrap_or(START);

    let edge_cost_fn = build_edge_cost_function(&map, TERRAIN);
    let dist_map = dijkstra_map(&map, &[goal], MOVE_SET, edge_cost_fn);

    if let Some(path) = path_from_search_map(&dist_map, start) {
        show_path_grid(
            &map,
            &path,
            format!("Dijkstra Path from {start:?} to {goal:?}"),
        );
    }

    if let Some(path) = a_star_weighted(
        &map,
        start,
        goal,
        MOVE_SET,
        build_edge_cost_function(&map, TERRAIN),
        |node, goal| MEAN_COST * (goal - node).distance_chebyshev(),
        HeuristicWeight::Dynamic(1.2),
    )? {
        show_path_grid(&map, &path, format!("A* Path from {start:?} to {goal:?}"));
    }

    Ok(())
}

fn is_cardinal_step(delta: PointDelta) -> bool {
    delta.dc == 0 || delta.dr == 0
}

fn show_grid(grid: &Grid<char>, caption: impl AsRef<str>) {
    print!("↓ {} ↓", caption.as_ref());
    grid.iter().enumerate().for_each(|(idx, (_, value))| {
        if idx % grid.size().width() == 0 {
            println!();
        }
        let icon = match *value {
            '.' => "·".dimmed(),
            '"' => "\"".green(),
            '~' => "~".bright_cyan(),
            '#' => "#".bright_red(),
            _ => unreachable!(),
        };
        print!("{icon}");
    });
    println!("\n");
}

fn show_path_grid(grid: &Grid<char>, path: &Path, caption: impl AsRef<str>) {
    println!("{:_^w$}", caption.as_ref(), w = grid.size().width());

    let stats = format!(
        "↓ Path Cost: {} | Path Steps: {} ↓",
        path.total_cost,
        path.points.len(),
    );
    println!("{stats:^w$}", w = grid.size().width());

    grid.iter().enumerate().for_each(|(idx, (point, value))| {
        if idx % grid.size().width() == 0 {
            println!();
        }

        let overlay = path
            .points
            .iter()
            .position(|&path_point| path_point == point)
            .map(|path_idx| match path_idx {
                0 => "S".bright_green().bold(),
                idx if idx == path.points.len() - 1 => "F".bright_purple().bold(),
                _ => "¤".bright_yellow(),
            });

        let icon = overlay.unwrap_or_else(|| match *value {
            '.' => "·".dimmed(),
            '"' => "¥".green().bold(),
            '~' => "≈".bright_cyan().bold(),
            '#' => "‡".black().on_truecolor(50, 50, 50),
            _ => unreachable!(),
        });
        print!("{icon}");
    });
    println!("\n");
}

fn read_map_into_grid(path: impl AsRef<std::path::Path>) -> GameResult<Grid<char>> {
    let contents = std::fs::read_to_string(path).unwrap();
    let rows = contents.lines().collect::<Vec<_>>();
    let width = rows[0].len() as u32;
    let height = rows.len() as u32;
    let grid_size = GridSize::new(width as usize, height as usize)?;
    Grid::from_vec(
        grid_size,
        rows.into_iter()
            .flat_map(|row| row.chars())
            .collect::<Vec<_>>(),
    )
}

// fn show_distance_grid(grid: &Grid<Option<u32>>, caption: impl AsRef<str>) {
//     print!("↓ {} ↓", caption.as_ref());
//     grid.iter().enumerate().for_each(|(idx, (_, value))| {
//         if idx % grid.size().width() == 0 {
//             println!();
//         }
//         print!("{:>3}", value.map_or("XX".to_string(), |d| d.to_string()));
//     });
//     println!("\n");
// }

fn build_edge_cost_function<T: Hash + Eq>(
    map: &Grid<T>,
    terrain_costs: impl IntoIterator<Item = (T, Option<u32>)>,
) -> impl Fn(Point, Point) -> Option<u32> {
    let terrain_cost = HashMap::<T, Option<u32>>::from_iter(terrain_costs);

    let edge_cost_fn = move |src: Point, dest: Point| -> Option<u32> {
        let movement_cost = if is_cardinal_step(src - dest) {
            CARDINAL_COST
        } else {
            DIAGONAL_COST
        };
        let t_cost = terrain_cost[&map[src]];
        t_cost.map(|c| c * movement_cost)
    };
    edge_cost_fn
}
