//! Pathfinding over [`Grid`] maps.
//!
//! Use [`dijkstra_map`] when several callers need paths to the same goal set,
//! then recover an individual path with [`path_from_search_map`]. Use [`a_star`]
//! when searching from one start point to one goal. Movement is selected with
//! [`MoveSet`], and the edge callback returns `None` for an impassable move.
//!
//! ```
//! use gametools::{Grid, GridSize, MoveSet, Point, a_star};
//!
//! let map = Grid::new(GridSize::new(3, 1).unwrap(), ()) .unwrap();
//! let path = a_star(
//!     &map,
//!     Point::new(0, 0),
//!     Point::new(2, 0),
//!     MoveSet::Cardinal,
//!     |_, _| Some(1),
//!     |from, to| (to - from).distance_taxicab(),
//! )
//! .unwrap();
//!
//! assert_eq!(path.total_cost, 2);
//! assert_eq!(path.points.front(), Some(&Point::new(0, 0)));
//! assert_eq!(path.points.back(), Some(&Point::new(2, 0)));
//! ```

use std::{collections::VecDeque, fmt::Debug};

use smallvec::SmallVec;

use crate::{
    GameResult, Grid, GridTopology, MinPriorityQ, Point, PointDelta, ensure,
    gameerror::PathfindingError,
};

/// The relative cost of movement along a path.
pub type Cost = u32;

/// Map structure used for holding movement costs and parent nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchMap {
    /// Cheapest known cost from each point to the goal set, or `None` when unreachable.
    pub costs: Grid<Option<Cost>>,
    /// The next point toward a goal for each reachable point.
    pub reached_from: Grid<Option<Point>>,
}

/// A constructed path from one point to another.
#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct Path {
    /// Points from the requested start through the reached goal, inclusive.
    pub points: VecDeque<Point>,
    /// Sum of the edge costs along [`Self::points`].
    pub total_cost: Cost,
}

/// The set of move directions the pathing algorithms must consider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveSet {
    /// North, west, east, and south one-cell moves.
    Cardinal,
    /// The four one-cell diagonal moves.
    Diagonal,
    /// All cardinal and diagonal one-cell moves.
    EightWay,
    /// Cardinal moves that land exactly `distance` cells away.
    CardinalJump {
        /// Number of cells between the origin and landing point.
        distance: u32,
    },
    /// Diagonal moves that land exactly `distance` cells away.
    DiagonalJump {
        /// Number of cells between the origin and landing point.
        distance: u32,
    },
    /// Moves described by caller-provided deltas, in either traversal order.
    Custom(&'static [PointDelta]),
}

/// Builds a Dijkstra search map from `goals` using bounded movement.
///
/// Although the search expands outward from the goals, `edge_cost` receives
/// `(from, to)` in the forward direction of the resulting path. Return `None`
/// for an impassable edge. When multiple equal-cost paths exist, their chosen
/// route is intentionally not stable.
///
/// # Panics
///
/// Panics when a goal is outside `map`.
///
/// # Examples
///
/// ```
/// use gametools::{Grid, GridSize, MoveSet, Point, dijkstra_map, path_from_search_map};
///
/// let map = Grid::new(GridSize::new(3, 1).unwrap(), ()) .unwrap();
/// let goal = Point::new(2, 0);
/// let search = dijkstra_map(&map, &[goal], MoveSet::Cardinal, |from, to| {
///     (to.col == from.col + 1).then_some(1)
/// });
/// assert_eq!(path_from_search_map(&search, Point::new(0, 0)).unwrap().total_cost, 2);
/// ```
pub fn dijkstra_map<T, F>(
    map: &Grid<T>,
    goals: &[Point],
    move_set: MoveSet,
    edge_cost: F,
) -> SearchMap
where
    F: Fn(Point, Point) -> Option<Cost>,
{
    dijkstra_map_with_topology(map, goals, move_set, GridTopology::Bounded, edge_cost)
}

/// Builds a Dijkstra search map using `topology` to resolve movement beyond
/// the map edges.
///
/// For [`GridTopology::Toroidal`], each goal is normalized before searching.
/// `edge_cost` still receives points in the forward path direction. See
/// [`dijkstra_map`] for tie-breaking behavior.
///
/// # Panics
///
/// Panics when a goal is outside `map` and `topology` is
/// [`GridTopology::Bounded`].
pub fn dijkstra_map_with_topology<T, F>(
    map: &Grid<T>,
    goals: &[Point],
    move_set: MoveSet,
    topology: GridTopology,
    edge_cost: F,
) -> SearchMap
where
    F: Fn(Point, Point) -> Option<Cost>,
{
    let mut frontier = MinPriorityQ::new();
    // note: GridSize already validated during `map` construction, so it cannot panic here
    let mut costs: Grid<Option<Cost>> =
        Grid::new(map.size(), None).expect("inherited map size must be valid");
    let mut reached_from = Grid::new(map.size(), None).expect("inherited map size must be valid");

    for &goal in goals {
        let goal = map
            .resolve_point(goal, topology)
            .expect("goal point must be in bounds for bounded pathfinding");
        frontier.push(goal, 0);
        costs[goal] = Some(0);
    }

    while let Some((current_point, path_cost)) = frontier.pop() {
        // prevents re-queueing of previously visited neighbors unless
        // this may be a better path through them
        if let Some(known_cost) = costs[current_point]
            && path_cost > known_cost
        {
            continue;
        }

        for neighbor in collect_neighbors(map, &move_set, topology, current_point) {
            let Some(edge_cost) = edge_cost(neighbor, current_point) else {
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

fn collect_neighbors<T>(
    map: &Grid<T>,
    move_set: &MoveSet,
    topology: GridTopology,
    point: Point,
) -> SmallVec<[Point; 8]> {
    let order_toggle = rand::random_bool(0.6);
    match (move_set, order_toggle) {
        (MoveSet::Cardinal, true) => map
            .neighbors(point, &PointDelta::CARDINALS, topology)
            .map(|(p, _)| p)
            .collect(),
        (MoveSet::Diagonal, true) => map
            .neighbors(point, &PointDelta::DIAGONALS, topology)
            .map(|(p, _)| p)
            .collect(),
        (MoveSet::EightWay, true) => map
            .neighbors(point, &PointDelta::DIAGONALS, topology)
            .chain(map.neighbors(point, &PointDelta::CARDINALS, topology))
            .map(|(p, _)| p)
            .collect(),
        (MoveSet::Cardinal, false) => map
            .neighbors(point, &PointDelta::CARDINALS, topology)
            .map(|(p, _)| p)
            .rev()
            .collect(),
        (MoveSet::Diagonal, false) => map
            .neighbors(point, &PointDelta::DIAGONALS, topology)
            .map(|(p, _)| p)
            .rev()
            .collect(),
        (MoveSet::EightWay, false) => map
            .neighbors(point, &PointDelta::CARDINALS, topology)
            .chain(map.neighbors(point, &PointDelta::DIAGONALS, topology))
            .map(|(p, _)| p)
            .rev()
            .collect(),
        (MoveSet::Custom(deltas), false) => map
            .neighbors(point, deltas, topology)
            .map(|(p, _)| p)
            .collect(),
        (MoveSet::Custom(deltas), true) => map
            .neighbors(point, deltas, topology)
            .map(|(p, _)| p)
            .rev()
            .collect(),
        (MoveSet::CardinalJump { distance }, _) => PointDelta::CARDINALS
            .iter()
            .filter_map(|delta| map.step_by(point, *delta, *distance, topology))
            .collect(),
        (MoveSet::DiagonalJump { distance }, _) => PointDelta::DIAGONALS
            .iter()
            .filter_map(|delta| map.step_by(point, *delta, *distance, topology))
            .collect(),
    }
}

/// Takes a result from dijkstra map and returns a best path from start to the
/// goal set by the search map.
///
/// Returns `None` when `start` is not reachable from the goal set.
///
/// # Panics
///
/// Panics when `start` is outside the search map.
#[must_use]
pub fn path_from_search_map(search_map: &SearchMap, start: Point) -> Option<Path> {
    search_map.costs[start]?;

    let mut path = VecDeque::from([start]);
    let mut current = start;
    while let Some(parent) = search_map.reached_from[current] {
        path.push_back(parent);
        current = parent;
    }
    Some(Path {
        points: path,
        total_cost: search_map.costs[start].map_or(0, |c| c),
    })
}

/// Returns an A* path from `start` to `goal` using bounded movement.
///
/// `edge_cost` receives `(from, to)` and returns `None` for an impassable move.
/// `heuristic` estimates the remaining cost from its first argument to its
/// second. An admissible heuristic preserves A*'s optimal-path guarantee.
/// Equal-cost paths may produce different routes between calls.
///
/// # Panics
///
/// Panics when `start` or `goal` is outside `map`.
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
    a_star_with_topology(
        map,
        start,
        goal,
        move_set,
        GridTopology::Bounded,
        edge_cost,
        heuristic,
    )
}

/// Returns a near-optimal path using `topology` to resolve movement beyond
/// the map edges.
///
/// For toroidal maps, supply a heuristic that accounts for wrapped distances
/// to retain A*'s usual efficiency. The start and goal are normalized with
/// `topology` before searching.
///
/// # Panics
///
/// Panics when `start` or `goal` is outside `map` and `topology` is
/// [`GridTopology::Bounded`].
pub fn a_star_with_topology<T, G, H>(
    map: &Grid<T>,
    start: Point,
    goal: Point,
    move_set: MoveSet,
    topology: GridTopology,
    edge_cost: G,
    heuristic: H,
) -> Option<Path>
where
    G: Fn(Point, Point) -> Option<Cost>,
    H: Fn(Point, Point) -> Cost,
{
    a_star_weighted_with_topology(
        map,
        start,
        goal,
        move_set,
        topology,
        edge_cost,
        heuristic,
        HeuristicWeight::Static(1.0),
    )
    .ok()?
}

/// Returns a weighted A* path from `start` to `goal` using bounded movement.
///
/// [`HeuristicWeight::Static`] accepts `0.0..=2.0`: `0.0` ignores the
/// heuristic, `1.0` is standard A*, and larger values increasingly favor the
/// heuristic over the known path cost. [`HeuristicWeight::Dynamic`] accepts
/// `1.0..=2.0` and uses the pxWD weighting function.
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
/// Invalid weights and unreachable goals both return `None`; use
/// [`a_star_weighted_with_topology`] when the invalid-weight error must be
/// distinguished from an unreachable goal.
///
/// # Panics
///
/// Panics when `start` or `goal` is outside `map`.
pub fn a_star_weighted<T, G, H>(
    map: &Grid<T>,
    start: Point,
    goal: Point,
    move_set: MoveSet,
    edge_cost: G,
    heuristic: H,
    weight: HeuristicWeight,
) -> Option<Path>
where
    G: Fn(Point, Point) -> Option<Cost>,
    H: Fn(Point, Point) -> Cost,
{
    a_star_weighted_with_topology(
        map,
        start,
        goal,
        move_set,
        GridTopology::Bounded,
        edge_cost,
        heuristic,
        weight,
    )
    .ok()?
}

/// Returns a weighted A* path using `topology` to resolve movement beyond the
/// map edges.
///
/// For toroidal maps, supply a heuristic that accounts for wrapped distances
/// to retain A*'s usual efficiency. The start and goal are normalized with
/// `topology` before searching.
///
/// # Panics
///
/// Panics when `start` or `goal` is outside `map` and `topology` is
/// [`GridTopology::Bounded`].
///
/// # Errors
///
/// Returns [`PathfindingError`] when `weight` is outside its valid range.
#[allow(clippy::too_many_arguments)] // Mirrors `a_star_weighted` with explicit topology.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
pub fn a_star_weighted_with_topology<T, G, H>(
    map: &Grid<T>,
    start: Point,
    goal: Point,
    move_set: MoveSet,
    topology: GridTopology,
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
    // note: this cannot panic because `map` already validated `Grid` dimensions
    // when constructed
    let mut costs = Grid::<Option<Cost>>::new(map.size(), None).expect("map.size() must be valid");
    let mut reached_from =
        Grid::<Option<Point>>::new(map.size(), None).expect("map.size() must be valid");

    let start = map
        .resolve_point(start, topology)
        .expect("start point must be in bounds for bounded pathfinding");
    let goal = map
        .resolve_point(goal, topology)
        .expect("goal point must be in bounds for bounded pathfinding");

    frontier.push((start, 0), (0, 0));
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

        for neighbor in collect_neighbors(map, &move_set, topology, point) {
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

            let priority = (priority, estimated_cost_left);

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
            points: VecDeque::from([start]),
            total_cost: 0,
        });
    }

    costs[goal]?;
    reached_from[goal]?;

    let mut path = VecDeque::from([goal]);
    let mut current = goal;
    while current != start {
        current = reached_from[current]?;
        path.push_front(current);
    }

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
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
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
/// Static values range from `0.0` (no heuristic) through `1.0` (standard A*)
/// to `2.0` (a more heuristic-biased A* search). Dynamic values use pxWD and
/// must be in the range `1.0..=2.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeuristicWeight {
    /// A fixed multiplier for the heuristic, in the range `0.0..=2.0`.
    Static(f32),
    /// A pxWD dynamic weight, in the range `1.0..=2.0`.
    Dynamic(f32),
}

#[cfg(test)]
mod tests {
    use super::{
        Cost, HeuristicWeight, MoveSet, a_star, a_star_weighted, a_star_with_topology,
        dijkstra_map, dijkstra_map_with_topology, path_from_search_map,
    };
    use crate::{GameResult, Grid, GridSize, GridTopology, Point, PointDelta};

    #[test]
    fn dijkstra_passes_edges_in_forward_direction() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 1)?, ())?;
        let start = Point::new(0, 0);
        let goal = Point::new(2, 0);

        let search_map = dijkstra_map(&map, &[goal], MoveSet::Cardinal, |src, dst| {
            if dst.col == src.col + 1 {
                Some(1)
            } else {
                None
            }
        });
        let path =
            path_from_search_map(&search_map, start).expect("directed forward path should exist");

        assert_eq!(path.points, vec![start, Point::new(1, 0), goal]);
        assert_eq!(path.total_cost, 2);
        Ok(())
    }

    #[test]
    fn dijkstra_returns_zero_length_path_at_goal() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 1)?, ())?;
        let goal = Point::new(2, 0);

        let search_map = dijkstra_map(&map, &[goal], MoveSet::Cardinal, |_, _| Some(1));
        let path = path_from_search_map(&search_map, goal)
            .expect("goal should have a zero-length path to itself");

        assert_eq!(path.points, vec![goal]);
        assert_eq!(path.total_cost, 0);
        Ok(())
    }

    #[test]
    fn dijkstra_uses_toroidal_neighbors() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 1)?, ())?;
        let start = Point::new(0, 0);
        let goal = Point::new(2, 0);

        let search_map = dijkstra_map_with_topology(
            &map,
            &[goal],
            MoveSet::Cardinal,
            GridTopology::Toroidal,
            |_, _| Some(1),
        );
        let path = path_from_search_map(&search_map, start).expect("wrapped path should exist");

        assert_eq!(path.points, vec![start, goal]);
        assert_eq!(path.total_cost, 1);
        Ok(())
    }

    #[test]
    fn a_star_tie_breaker_does_not_outweigh_f_cost() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 2)?, ())?;
        let start = Point::new(0, 0);
        let goal = Point::new(2, 0);
        let detour = Point::new(0, 1);

        const MOVES: &[PointDelta] = &[
            PointDelta::new(0, 1),
            PointDelta::new(1, 0),
            PointDelta::new(2, 0),
            PointDelta::new(2, -1),
        ];

        let path = a_star(
            &map,
            start,
            goal,
            MoveSet::Custom(MOVES),
            |src, dst| match (src, dst) {
                (p, q) if p == start && q == detour => Some(1),
                (p, q) if p == detour && q == goal => Some(2_000),
                (p, q) if p == start && q == goal => Some(2_002),
                _ => None,
            },
            |src, _| if src == detour { 2_000 } else { 0 },
        )
        .expect("path should exist");

        assert_eq!(path.points, vec![start, detour, goal]);
        assert_eq!(path.total_cost, 2_001);
        Ok(())
    }

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
    fn a_star_uses_toroidal_neighbors() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 1)?, ())?;
        let start = Point::new(0, 0);
        let goal = Point::new(2, 0);

        let path = a_star_with_topology(
            &map,
            start,
            goal,
            MoveSet::Cardinal,
            GridTopology::Toroidal,
            |_, _| Some(1),
            |_, _| 0,
        )
        .expect("wrapped path should exist");

        assert_eq!(path.points, vec![start, goal]);
        assert_eq!(path.total_cost, 1);
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
        )
        .unwrap();

        assert_eq!(path.points, vec![start, Point::new(1, 0), goal]);
        assert_eq!(path.total_cost, 2);
        Ok(())
    }
}
