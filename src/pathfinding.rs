use std::fmt::Debug;

use smallvec::SmallVec;

use crate::{
    GameResult, Grid, MinPriorityQ, Point, PointDelta, ensure, gameerror::PathfindingError,
};

/// The relative cost of movement along a path.
pub type Cost = u32;

/// Map structure used for holding movement costs and parent nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchMap {
    pub costs: Grid<Option<Cost>>,
    pub reached_from: Grid<Option<Point>>,
}

/// A constructed path from one point to another.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub points: Vec<Point>,
    pub total_cost: Cost,
}

/// The set of move directions the pathing algorithms must consider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveSet {
    Cardinal,
    Diagonal,
    EightWay,
    CardinalJump { distance: u32 },
    DiagonalJump { distance: u32 },
    Custom(&'static [PointDelta]),
}

pub fn dijkstra_map<T, F>(
    map: &Grid<T>,
    goals: &[Point],
    move_set: MoveSet,
    edge_cost: F,
) -> SearchMap
where
    F: Fn(Point, Point) -> Option<Cost>,
{
    let mut frontier = MinPriorityQ::new();
    let mut costs: Grid<Option<Cost>> =
        Grid::new(map.size(), None).expect("inherited map size must be valid");
    let mut reached_from = Grid::new(map.size(), None).expect("inherited map size must be valid");

    for goal in goals {
        frontier.push(*goal, 0);
        costs[*goal] = Some(0);
    }

    while let Some((current_point, path_cost)) = frontier.pop() {
        // prevent re-queueing of previously visited neighbors unless
        // this may be a better path through them
        if let Some(known_cost) = costs[current_point]
            && path_cost > known_cost
        {
            continue;
        }

        for neighbor in collect_neighbors(map, &move_set, current_point) {
            let Some(edge_cost) = edge_cost(current_point, neighbor) else {
                continue;
            };
            let new_cost = path_cost + edge_cost;
            // only update neighbor if we found a lower cost path to it, or it was not yet visited
            if costs[neighbor].is_none_or(|old_cost| new_cost < old_cost) {
                costs[neighbor] = Some(new_cost);
                reached_from[neighbor] = Some(current_point);
                frontier.push(neighbor, new_cost);
            }
        }
    }
    SearchMap {
        costs,
        reached_from,
    }
}

fn collect_neighbors<T>(map: &Grid<T>, move_set: &MoveSet, point: Point) -> SmallVec<[Point; 8]> {
    let order_toggle = rand::random_bool(0.6);
    match (move_set, order_toggle) {
        (MoveSet::Cardinal, true) => map.cardinal_neighbors(point).map(|(p, _)| p).collect(),
        (MoveSet::Diagonal, true) => map.diagonal_neighbors(point).map(|(p, _)| p).collect(),
        (MoveSet::EightWay, true) => map
            .diagonal_neighbors(point)
            .chain(map.cardinal_neighbors(point))
            .map(|(p, _)| p)
            .collect(),
        (MoveSet::Cardinal, false) => map
            .cardinal_neighbors(point)
            .map(|(p, _)| p)
            .rev()
            .collect(),
        (MoveSet::Diagonal, false) => map
            .diagonal_neighbors(point)
            .map(|(p, _)| p)
            .rev()
            .collect(),
        (MoveSet::EightWay, false) => map
            .cardinal_neighbors(point)
            .chain(map.diagonal_neighbors(point))
            .map(|(p, _)| p)
            .rev()
            .collect(),
        (MoveSet::Custom(deltas), false) => deltas
            .iter()
            .filter_map(|d| {
                let neighbor = point + *d;
                map.contains_point(neighbor).then_some(neighbor)
            })
            .collect(),
        (MoveSet::Custom(deltas), true) => deltas
            .iter()
            .rev()
            .filter_map(|d| {
                let neighbor = point + *d;
                map.contains_point(neighbor).then_some(neighbor)
            })
            .collect(),
        (MoveSet::CardinalJump { distance }, _) => PointDelta::CARDINALS
            .iter()
            .map(|delta| point + *delta * *distance)
            .filter(|neighbor| map.contains_point(*neighbor))
            .collect(),
        (MoveSet::DiagonalJump { distance }, _) => PointDelta::DIAGONALS
            .iter()
            .map(|delta| point + *delta * *distance)
            .filter(|neighbor| map.contains_point(*neighbor))
            .collect(),
    }
}

/// Takes a result from dijkstra map and returns a best path from start to the
/// goal set by the search map.
pub fn path_from_search_map(search_map: &SearchMap, start: Point) -> Option<Path> {
    let nodes_searched = search_map
        .reached_from
        .iter()
        .filter(|(_, v)| v.is_some())
        .count();
    dbg!(nodes_searched);

    search_map.reached_from[start]?;

    let mut path = vec![start];
    let mut current = start;
    while let Some(parent) = search_map.reached_from[current] {
        path.push(parent);
        current = parent;
    }
    Some(Path {
        points: path,
        total_cost: search_map.costs[start].map_or(0, |c| c),
    })
}

/// Returns a near-optimal path from start to goal using the original A* algorithm.
/// - 'edge_cost' is a function that returns the cost (distance, turns, etc.) to move from one point to another
/// - 'heuristic' is a function that estimates the cost to reach the goal from a given point
pub fn a_star<T, G, H>(
    map: &Grid<T>,
    start: Point,
    goal: Point,
    move_set: MoveSet,
    edge_cost: G,
    heuristic: H,
) -> Option<Path>
where
    G: Fn(Point, Point) -> Option<Cost>,
    H: Fn(Point, Point) -> Cost,
{
    a_star_weighted(
        map,
        start,
        goal,
        move_set,
        edge_cost,
        heuristic,
        HeuristicWeight::Static(1.0),
    )
    .ok()?
}

/// Returns a near-optimal path from start to goal using the A* algorithm with a weight factor.
///
/// Weight must be >= 1.0 and <= 2.0. At weight = 1.0, it is the same as standard A*. At 2.0, it behaves
/// like greedy Best First Search. For weights inbetween, it *leans* more toward one or the other.
///
/// *Note: if weight were 0.0, this would be theoretically equivalent to Dijkstra, but values < 1.0 can
/// cause an overflow in the weighted fitness calculation.*
///
/// # Parameters
///
/// * `map` - The grid map to search.
/// * `start` - The starting point.
/// * `goal` - The goal point.
/// * `move_set` - The set of allowed moves.
/// * `edge_cost` - The cost function for edge traversal.
/// * `heuristic` - The heuristic function for estimating the cost to the goal.
/// * `weight` - The weight factor for the heuristic. See [`HeuristicWeight`]
///
/// # Errors
///
/// Returns an error if the weight is outside the valid range.
pub fn a_star_weighted<T, G, H>(
    map: &Grid<T>,
    start: Point,
    goal: Point,
    move_set: MoveSet,
    edge_cost: G,
    heuristic: H,
    weight: HeuristicWeight,
) -> GameResult<Option<Path>>
where
    G: Fn(Point, Point) -> Option<Cost>,
    H: Fn(Point, Point) -> Cost,
{
    match weight {
        HeuristicWeight::Static(w) => {
            ensure!(
                (0.0..=2.0).contains(&w),
                PathfindingError::InvalidStaticWeight(w)
            );
        }
        HeuristicWeight::Dynamic(w) => {
            ensure!(
                (1.0..=2.0).contains(&w),
                PathfindingError::InvalidDynamicWeight(w)
            );
        }
    }

    let mut frontier = MinPriorityQ::new();
    let mut costs = Grid::<Option<Cost>>::new(map.size(), None).expect("map.size() must be valid");
    let mut reached_from =
        Grid::<Option<Point>>::new(map.size(), None).expect("map.size() must be valid");

    frontier.push((start, 0), 0);
    costs[start] = Some(0);

    while let Some(((point, path_cost), _priority)) = frontier.pop() {
        if let Some(known_cost) = costs[point]
            && path_cost > known_cost
        {
            continue; /* skip this point, we already have a better path to it */
        }

        if point == goal {
            break; /* path complete */
        }

        for neighbor in collect_neighbors(map, &move_set, point) {
            let Some(edge_cost) = edge_cost(point, neighbor) else {
                continue;
            };
            let new_cost = /* g(nearby_point) */ path_cost + edge_cost;
            let estimated_cost_left = /* h(nearby_point) */ heuristic(neighbor, goal);
            let priority = /* f(nearby_point) */ match weight {
                HeuristicWeight::Dynamic(weight) => {
                    pxwd_dynamic_weight(weight, new_cost, estimated_cost_left)
                }
                HeuristicWeight::Static(weight) => {
                    new_cost + (weight * estimated_cost_left as f32) as Cost
                }
            };

            // tiebreaker applied to priority
            let priority: u64 = u64::from(priority) * 1000 + u64::from(estimated_cost_left);

            if costs[neighbor].is_none_or(|old_cost| new_cost < old_cost) {
                costs[neighbor] = Some(new_cost);
                frontier.push((neighbor, new_cost), priority);
                reached_from[neighbor] = Some(point);
            }
        }
    }

    Ok(path_from_forward_search(&costs, &reached_from, start, goal))
}

fn path_from_forward_search(
    costs: &Grid<Option<Cost>>,
    reached_from: &Grid<Option<Point>>,
    start: Point,
    goal: Point,
) -> Option<Path> {
    if start == goal {
        return Some(Path {
            points: vec![start],
            total_cost: 0,
        });
    }

    costs[goal]?;
    reached_from[goal]?;

    let mut path = vec![goal];
    let mut current = goal;
    while current != start {
        current = reached_from[current]?;
        path.push(current);
    }
    path.reverse();

    Some(Path {
        points: path,
        total_cost: costs[goal].map_or(0, |c| c),
    })
}

/// Returns a dynamically weighted path cost for A* algorithm.
///
/// Calculates a weighted path cost using the pxWD dynamic weighting function from Chen and
/// Sturtevant 2021, which I found through Amit Patel's "Thoughts on Pathfinding" site.
///
/// Ultimately, the effect is that the algorithm is standard A* until about ½ way to the goal,
/// at which point the weight increases the contribution of the heuristic in the path fitness calculation.
fn pxwd_dynamic_weight(weight: f32, new_cost: Cost, estimated_cost_left: Cost) -> Cost {
    if new_cost < estimated_cost_left {
        new_cost + estimated_cost_left
    } else {
        ((new_cost as f32 + (2.0 * weight - 1.0) * estimated_cost_left as f32) / weight) as u32
    }
}

/// The weight factor for the heuristic in the A* algorithm.
///
/// This can be a static value or a dynamic weight calculated using the pxWD function.
/// Static values may range from 0.0 (no heuristic, == Dijkstra) to 1.0 (standard A*) to 2.0 (BFS).
/// Dynamic values are used by the pxWD function and must be in the range (1.0..=2.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeuristicWeight {
    Static(f32),
    Dynamic(f32),
}

#[cfg(test)]
mod tests {
    use super::{Cost, HeuristicWeight, MoveSet, a_star, a_star_weighted};
    use crate::{GameResult, Grid, GridSize, Point};

    #[test]
    fn a_star_passes_edges_in_forward_direction() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 1)?, ())?;
        let start = Point::new(0, 0);
        let goal = Point::new(2, 0);

        let path = a_star(
            &map,
            start,
            goal,
            MoveSet::Cardinal,
            |src, dst| {
                if dst.col == src.col + 1 {
                    Some(1)
                } else {
                    None
                }
            },
            |src, dst| (dst - src).distance_taxicab(),
        )
        .expect("directed forward path should exist");

        assert_eq!(path.points, vec![start, Point::new(1, 0), goal]);
        assert_eq!(path.total_cost, 2);
        Ok(())
    }

    #[test]
    fn a_star_allows_pathing_from_an_occupied_start_tile() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 1)?, ())?;
        let start = Point::new(0, 0);
        let goal = Point::new(2, 0);

        let path = a_star_weighted(
            &map,
            start,
            goal,
            MoveSet::Cardinal,
            |_, dst| {
                if dst == start { None } else { Some(1 as Cost) }
            },
            |src, dst| (dst - src).distance_taxicab(),
            HeuristicWeight::Static(1.0),
        )?
        .expect("occupied start should not block leaving the start");

        assert_eq!(path.points, vec![start, Point::new(1, 0), goal]);
        assert_eq!(path.total_cost, 2);
        Ok(())
    }
}
