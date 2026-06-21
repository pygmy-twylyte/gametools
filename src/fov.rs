//! Field-of-view algorithms for point-addressed [`Grid`](crate::Grid) maps.
//!
//! The algorithms in this module take a caller-provided opacity function so the
//! map can store any tile type. Results are represented as a [`Grid<bool>`],
//! where `true` means the cell is visible from the source point.

use std::f64::consts::TAU;

use crate::{Grid, GridSize, Point};

/// A grid containing field-of-view visibility flags.
pub type FovMap = Grid<bool>;

/// Calculates a field of view by casting rays to the bounded map perimeter.
///
/// Blocking cells are marked visible and stop the ray that reached them.
/// `radius` is a Chebyshev-distance limit; `None` scans to the map boundary.
#[must_use]
pub fn perimeter_raycasting<T, F>(
    map: &Grid<T>,
    source: Point,
    radius: Option<u32>,
    blocks_vision: F,
) -> FovMap
where
    F: Fn(Point, &T) -> bool,
{
    let mut visible = Grid::new(map.size(), false).expect("inherited map size must be valid");
    perimeter_raycasting_into(map, &mut visible, source, radius, blocks_vision);
    visible
}

/// Writes a perimeter-raycast field of view into an existing visibility grid.
///
/// Reusing `visible` avoids repeated allocation in hot loops. The visibility
/// grid must have the same size as `map`.
pub fn perimeter_raycasting_into<T, F>(
    map: &Grid<T>,
    visible: &mut FovMap,
    source: Point,
    radius: Option<u32>,
    blocks_vision: F,
) where
    F: Fn(Point, &T) -> bool,
{
    assert_eq!(
        map.size(),
        visible.size(),
        "visibility grid size must match map size"
    );
    clear_visibility(visible);

    let Some(radius) = effective_radius_for_size(map.size(), source, radius) else {
        return;
    };

    visible[source] = true;
    if radius == 0 {
        return;
    }

    let bounds = Bounds::for_radius(map.size(), source, radius);
    for col in bounds.min_col..=bounds.max_col {
        trace_ray(
            map,
            visible,
            source,
            Point::new(col, bounds.min_row),
            radius,
            &blocks_vision,
        );
        if bounds.max_row != bounds.min_row {
            trace_ray(
                map,
                visible,
                source,
                Point::new(col, bounds.max_row),
                radius,
                &blocks_vision,
            );
        }
    }

    for row in (bounds.min_row + 1)..bounds.max_row {
        trace_ray(
            map,
            visible,
            source,
            Point::new(bounds.min_col, row),
            radius,
            &blocks_vision,
        );
        if bounds.max_col != bounds.min_col {
            trace_ray(
                map,
                visible,
                source,
                Point::new(bounds.max_col, row),
                radius,
                &blocks_vision,
            );
        }
    }
}

/// Calculates a field of view with the classic recursive shadowcasting algorithm.
///
/// Recursive shadowcasting tends to perform well for small or enclosed maps
/// because it only visits cells inside visible angular bands. `radius` is a
/// Chebyshev-distance limit; `None` scans to the map boundary.
#[must_use]
pub fn recursive_shadowcasting<T, F>(
    map: &Grid<T>,
    source: Point,
    radius: Option<u32>,
    blocks_vision: F,
) -> FovMap
where
    F: Fn(Point, &T) -> bool,
{
    let mut visible = Grid::new(map.size(), false).expect("inherited map size must be valid");
    recursive_shadowcasting_into(map, &mut visible, source, radius, blocks_vision);
    visible
}

/// Writes a recursive-shadowcasting field of view into an existing visibility grid.
///
/// Reusing `visible` avoids repeated allocation in hot loops. The visibility
/// grid must have the same size as `map`.
pub fn recursive_shadowcasting_into<T, F>(
    map: &Grid<T>,
    visible: &mut FovMap,
    source: Point,
    radius: Option<u32>,
    blocks_vision: F,
) where
    F: Fn(Point, &T) -> bool,
{
    assert_eq!(
        map.size(),
        visible.size(),
        "visibility grid size must match map size"
    );
    clear_visibility(visible);

    let Some(radius) = effective_radius_for_size(map.size(), source, radius) else {
        return;
    };

    visible[source] = true;
    if radius == 0 {
        return;
    }

    for octant in OCTANTS {
        cast_shadow_octant(
            map,
            visible,
            source,
            radius,
            1,
            1.0,
            0.0,
            octant,
            &blocks_vision,
        );
    }
}

/// Calculates a rectangle-based field of view, building the blocker cache first.
///
/// This is a convenience wrapper around [`RectangleFov`]. Prefer
/// [`RectangleFov`] directly when the blocking terrain is reused across frames.
#[must_use]
pub fn rectangle_based_fov<T, F>(
    map: &Grid<T>,
    source: Point,
    radius: Option<u32>,
    blocks_vision: F,
) -> FovMap
where
    F: Fn(Point, &T) -> bool,
{
    RectangleFov::new(map, blocks_vision).visible_from(source, radius)
}

/// Writes a rectangle-based field of view into an existing visibility grid.
///
/// This convenience wrapper rebuilds the rectangle cache. Prefer
/// [`RectangleFov::visible_from_into`] when the blocking terrain is reused.
pub fn rectangle_based_fov_into<T, F>(
    map: &Grid<T>,
    visible: &mut FovMap,
    source: Point,
    radius: Option<u32>,
    blocks_vision: F,
) where
    F: Fn(Point, &T) -> bool,
{
    RectangleFov::new(map, blocks_vision).visible_from_into(visible, source, radius);
}

/// An inclusive rectangle of vision-blocking cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockingRect {
    /// The top-left blocking cell in the rectangle.
    pub min: Point,
    /// The bottom-right blocking cell in the rectangle.
    pub max: Point,
}

impl BlockingRect {
    #[must_use]
    fn from_bounds(min_col: i32, min_row: i32, max_col: i32, max_row: i32) -> Self {
        Self {
            min: Point::new(min_col, min_row),
            max: Point::new(max_col, max_row),
        }
    }

    /// Returns the rectangle width in cells.
    #[must_use]
    pub fn width(&self) -> u32 {
        (self.max.col - self.min.col + 1) as u32
    }

    /// Returns the rectangle height in cells.
    #[must_use]
    pub fn height(&self) -> u32 {
        (self.max.row - self.min.row + 1) as u32
    }

    /// Returns whether `point` is inside this blocking rectangle.
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        (self.min.col..=self.max.col).contains(&point.col)
            && (self.min.row..=self.max.row).contains(&point.row)
    }

    fn left(&self) -> f64 {
        f64::from(self.min.col)
    }

    fn right(&self) -> f64 {
        f64::from(self.max.col + 1)
    }

    fn top(&self) -> f64 {
        f64::from(self.min.row)
    }

    fn bottom(&self) -> f64 {
        f64::from(self.max.row + 1)
    }

    fn vertices(&self) -> [Vec2; 4] {
        [
            Vec2::new(self.left(), self.top()),
            Vec2::new(self.right(), self.top()),
            Vec2::new(self.right(), self.bottom()),
            Vec2::new(self.left(), self.bottom()),
        ]
    }

    fn contains_continuous(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }
}

/// Preprocessed vision-blocking rectangles for rectangle-based FOV.
///
/// This follows the rectangle representation described by Debenham and
/// Solis-Oba's rectangle-based FOV algorithm: blocking cells are grouped into
/// rectangles once, then each FOV calculation marks the row-major shadowed
/// cells cast by those rectangles.
#[derive(Debug, Clone, PartialEq)]
pub struct RectangleFov {
    size: GridSize,
    rectangles: Vec<BlockingRect>,
}

impl RectangleFov {
    /// Builds a reusable rectangle cache from the blocking cells in `map`.
    ///
    /// The grouping is greedy and row-major: it extracts maximal unvisited
    /// rectangles of blocking cells without adding dependencies or requiring a
    /// more expensive minimal rectilinear decomposition step.
    #[must_use]
    pub fn new<T, F>(map: &Grid<T>, blocks_vision: F) -> Self
    where
        F: Fn(Point, &T) -> bool,
    {
        let mut visited = Grid::new(map.size(), false).expect("inherited map size must be valid");
        let mut rectangles = Vec::new();

        for row in 0..height_i32(map.size()) {
            for col in 0..width_i32(map.size()) {
                let point = Point::new(col, row);
                if visited[point] || !blocks_vision(point, &map[point]) {
                    continue;
                }

                let mut max_col = col;
                while max_col + 1 < width_i32(map.size()) {
                    let next = Point::new(max_col + 1, row);
                    if visited[next] || !blocks_vision(next, &map[next]) {
                        break;
                    }
                    max_col += 1;
                }

                let mut max_row = row;
                'rows: while max_row + 1 < height_i32(map.size()) {
                    let next_row = max_row + 1;
                    for rect_col in col..=max_col {
                        let next = Point::new(rect_col, next_row);
                        if visited[next] || !blocks_vision(next, &map[next]) {
                            break 'rows;
                        }
                    }
                    max_row = next_row;
                }

                for rect_row in row..=max_row {
                    for rect_col in col..=max_col {
                        visited[Point::new(rect_col, rect_row)] = true;
                    }
                }

                rectangles.push(BlockingRect::from_bounds(col, row, max_col, max_row));
            }
        }

        Self {
            size: map.size(),
            rectangles,
        }
    }

    /// Returns the size of the map this FOV cache was built from.
    #[must_use]
    pub fn size(&self) -> GridSize {
        self.size
    }

    /// Returns the preprocessed blocking rectangles.
    #[must_use]
    pub fn blocking_rectangles(&self) -> &[BlockingRect] {
        &self.rectangles
    }

    /// Calculates a field of view from `source`.
    ///
    /// `radius` is a Chebyshev-distance limit; `None` scans to the map boundary.
    #[must_use]
    pub fn visible_from(&self, source: Point, radius: Option<u32>) -> FovMap {
        let mut visible = Grid::new(self.size, false).expect("inherited map size must be valid");
        self.visible_from_into(&mut visible, source, radius);
        visible
    }

    /// Writes a field of view from `source` into an existing visibility grid.
    ///
    /// Reusing `visible` avoids repeated allocation in hot loops. The visibility
    /// grid must have the same size as the map used to build this cache.
    pub fn visible_from_into(&self, visible: &mut FovMap, source: Point, radius: Option<u32>) {
        assert_eq!(
            self.size,
            visible.size(),
            "visibility grid size must match rectangle FOV size"
        );
        clear_visibility(visible);

        let Some(radius) = effective_radius_for_size(self.size, source, radius) else {
            return;
        };

        fill_visible_radius(visible, source, radius);
        if radius == 0 {
            return;
        }

        let source_center = cell_center(source);
        for rectangle in &self.rectangles {
            if rectangle.contains(source) {
                continue;
            }
            let Some(shadow) = RectangleShadow::new(source_center, *rectangle) else {
                continue;
            };
            let Some(bounds) =
                shadow.candidate_bounds(self.size, source_center, source, radius, *rectangle)
            else {
                continue;
            };

            for row in bounds.min_row..=bounds.max_row {
                for col in bounds.min_col..=bounds.max_col {
                    let point = Point::new(col, row);
                    if visible[point] && shadow.cell_fully_occluded(point, *rectangle) {
                        visible[point] = false;
                    }
                }
            }
        }

        visible[source] = true;
    }
}

#[derive(Debug, Clone, Copy)]
struct Octant {
    xx: i32,
    xy: i32,
    yx: i32,
    yy: i32,
}

const OCTANTS: [Octant; 8] = [
    Octant {
        xx: 1,
        xy: 0,
        yx: 0,
        yy: 1,
    },
    Octant {
        xx: 0,
        xy: 1,
        yx: 1,
        yy: 0,
    },
    Octant {
        xx: 0,
        xy: -1,
        yx: 1,
        yy: 0,
    },
    Octant {
        xx: -1,
        xy: 0,
        yx: 0,
        yy: 1,
    },
    Octant {
        xx: -1,
        xy: 0,
        yx: 0,
        yy: -1,
    },
    Octant {
        xx: 0,
        xy: -1,
        yx: -1,
        yy: 0,
    },
    Octant {
        xx: 0,
        xy: 1,
        yx: -1,
        yy: 0,
    },
    Octant {
        xx: 1,
        xy: 0,
        yx: 0,
        yy: -1,
    },
];

#[allow(clippy::too_many_arguments)]
fn cast_shadow_octant<T, F>(
    map: &Grid<T>,
    visible: &mut FovMap,
    source: Point,
    radius: i32,
    row: i32,
    mut start_slope: f64,
    end_slope: f64,
    octant: Octant,
    blocks_vision: &F,
) where
    F: Fn(Point, &T) -> bool,
{
    if start_slope < end_slope {
        return;
    }

    for distance in row..=radius {
        let mut blocked = false;
        let mut next_start_slope = start_slope;

        for delta_col in -distance..=0 {
            let delta_row = -distance;
            let current = Point::new(
                source.col + delta_col * octant.xx + delta_row * octant.xy,
                source.row + delta_col * octant.yx + delta_row * octant.yy,
            );

            let left_slope = (f64::from(delta_col) - 0.5) / (f64::from(delta_row) + 0.5);
            let right_slope = (f64::from(delta_col) + 0.5) / (f64::from(delta_row) - 0.5);

            if start_slope < right_slope {
                continue;
            }
            if end_slope > left_slope {
                break;
            }

            let opaque = if map.contains_point(current) {
                visible[current] = true;
                blocks_vision(current, &map[current])
            } else {
                false
            };

            if blocked {
                if opaque {
                    next_start_slope = right_slope;
                } else {
                    blocked = false;
                    start_slope = next_start_slope;
                }
            } else if opaque && distance < radius {
                blocked = true;
                cast_shadow_octant(
                    map,
                    visible,
                    source,
                    radius,
                    distance + 1,
                    start_slope,
                    left_slope,
                    octant,
                    blocks_vision,
                );
                next_start_slope = right_slope;
            }
        }

        if blocked {
            break;
        }
    }
}

fn trace_ray<T, F>(
    map: &Grid<T>,
    visible: &mut FovMap,
    source: Point,
    target: Point,
    radius: i32,
    blocks_vision: &F,
) where
    F: Fn(Point, &T) -> bool,
{
    trace_line(source, target, |point| {
        if !map.contains_point(point) || !within_radius(source, point, radius) {
            return false;
        }

        visible[point] = true;
        point == source || !blocks_vision(point, &map[point])
    });
}

fn trace_line<F>(source: Point, target: Point, mut visit: F)
where
    F: FnMut(Point) -> bool,
{
    let mut point = source;
    let dx = (i64::from(target.col) - i64::from(source.col)).abs();
    let dy = -(i64::from(target.row) - i64::from(source.row)).abs();
    let step_col = if source.col < target.col { 1 } else { -1 };
    let step_row = if source.row < target.row { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        if !visit(point) || point == target {
            break;
        }

        let doubled_error = error * 2;
        if doubled_error >= dy {
            error += dy;
            point.col += step_col;
        }
        if doubled_error <= dx {
            error += dx;
            point.row += step_row;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RectangleShadow {
    source: Vec2,
    start: Vec2,
    end: Vec2,
}

impl RectangleShadow {
    fn new(source: Vec2, rectangle: BlockingRect) -> Option<Self> {
        if rectangle.contains_continuous(source) {
            return None;
        }

        let mut vertices = rectangle.vertices().map(|vertex| AnglePoint {
            angle: (vertex - source).angle(),
            point: vertex,
        });
        vertices.sort_by(|a, b| a.angle.total_cmp(&b.angle));

        let mut largest_gap = f64::NEG_INFINITY;
        let mut gap_index = 0;
        for index in 0..vertices.len() {
            let next = (index + 1) % vertices.len();
            let next_angle = if next == 0 {
                vertices[next].angle + TAU
            } else {
                vertices[next].angle
            };
            let gap = next_angle - vertices[index].angle;
            if gap > largest_gap {
                largest_gap = gap;
                gap_index = index;
            }
        }

        let start = vertices[(gap_index + 1) % vertices.len()].point - source;
        let end = vertices[gap_index].point - source;
        (start.cross(end) > EPSILON).then_some(Self { source, start, end })
    }

    fn candidate_bounds(
        &self,
        size: GridSize,
        source_center: Vec2,
        source: Point,
        radius: i32,
        rectangle: BlockingRect,
    ) -> Option<Bounds> {
        let mut bounds = Bounds::for_radius(size, source, radius);

        if source_center.x < rectangle.left() {
            bounds.min_col = bounds.min_col.max(rectangle.min.col);
        } else if source_center.x > rectangle.right() {
            bounds.max_col = bounds.max_col.min(rectangle.max.col);
        }

        if source_center.y < rectangle.top() {
            bounds.min_row = bounds.min_row.max(rectangle.min.row);
        } else if source_center.y > rectangle.bottom() {
            bounds.max_row = bounds.max_row.min(rectangle.max.row);
        }

        (!bounds.is_empty()).then_some(bounds)
    }

    fn cell_fully_occluded(&self, point: Point, rectangle: BlockingRect) -> bool {
        if rectangle.contains(point) {
            return false;
        }

        cell_corners(point).into_iter().all(|corner| {
            self.contains_strict(corner) && segment_intersects_rect(self.source, corner, rectangle)
        })
    }

    fn contains_strict(&self, point: Vec2) -> bool {
        let point = point - self.source;
        self.start.cross(point) > EPSILON && point.cross(self.end) > EPSILON
    }
}

#[derive(Debug, Clone, Copy)]
struct AnglePoint {
    angle: f64,
    point: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }

    fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

const EPSILON: f64 = 1e-9;

fn segment_intersects_rect(start: Vec2, end: Vec2, rectangle: BlockingRect) -> bool {
    let direction = end - start;
    let mut min_t = 0.0;
    let mut max_t = 1.0;

    clip_segment(
        -direction.x,
        start.x - rectangle.left(),
        &mut min_t,
        &mut max_t,
    ) && clip_segment(
        direction.x,
        rectangle.right() - start.x,
        &mut min_t,
        &mut max_t,
    ) && clip_segment(
        -direction.y,
        start.y - rectangle.top(),
        &mut min_t,
        &mut max_t,
    ) && clip_segment(
        direction.y,
        rectangle.bottom() - start.y,
        &mut min_t,
        &mut max_t,
    ) && max_t >= -EPSILON
        && min_t <= 1.0 + EPSILON
}

fn clip_segment(denominator: f64, numerator: f64, min_t: &mut f64, max_t: &mut f64) -> bool {
    if denominator.abs() < EPSILON {
        return numerator >= -EPSILON;
    }

    let ratio = numerator / denominator;
    if denominator < 0.0 {
        if ratio > *max_t {
            return false;
        }
        if ratio > *min_t {
            *min_t = ratio;
        }
    } else {
        if ratio < *min_t {
            return false;
        }
        if ratio < *max_t {
            *max_t = ratio;
        }
    }

    true
}

fn cell_center(point: Point) -> Vec2 {
    Vec2::new(f64::from(point.col) + 0.5, f64::from(point.row) + 0.5)
}

fn cell_corners(point: Point) -> [Vec2; 4] {
    let left = f64::from(point.col);
    let right = f64::from(point.col + 1);
    let top = f64::from(point.row);
    let bottom = f64::from(point.row + 1);

    [
        Vec2::new(left, top),
        Vec2::new(right, top),
        Vec2::new(right, bottom),
        Vec2::new(left, bottom),
    ]
}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min_col: i32,
    max_col: i32,
    min_row: i32,
    max_row: i32,
}

impl Bounds {
    fn for_radius(size: GridSize, source: Point, radius: i32) -> Self {
        Self {
            min_col: (source.col - radius).max(0),
            max_col: (source.col + radius).min(width_i32(size) - 1),
            min_row: (source.row - radius).max(0),
            max_row: (source.row + radius).min(height_i32(size) - 1),
        }
    }

    fn is_empty(&self) -> bool {
        self.min_col > self.max_col || self.min_row > self.max_row
    }
}

fn clear_visibility(visible: &mut FovMap) {
    for (_, value) in visible.iter_mut() {
        *value = false;
    }
}

fn fill_visible_radius(visible: &mut FovMap, source: Point, radius: i32) {
    for (point, value) in visible.iter_mut() {
        *value = within_radius(source, point, radius);
    }
}

fn effective_radius_for_size(size: GridSize, source: Point, radius: Option<u32>) -> Option<i32> {
    if !point_in_size(size, source) {
        return None;
    }

    let full_radius = max_radius_to_edge(size, source);
    let requested = radius
        .and_then(|value| i32::try_from(value).ok())
        .unwrap_or(i32::MAX);
    Some(requested.min(full_radius))
}

fn max_radius_to_edge(size: GridSize, source: Point) -> i32 {
    let max_col = width_i32(size) - 1;
    let max_row = height_i32(size) - 1;
    source
        .col
        .max(max_col - source.col)
        .max(source.row)
        .max(max_row - source.row)
}

fn point_in_size(size: GridSize, point: Point) -> bool {
    point.col >= 0 && point.row >= 0 && point.col < width_i32(size) && point.row < height_i32(size)
}

fn within_radius(source: Point, point: Point, radius: i32) -> bool {
    (point.col - source.col)
        .abs()
        .max((point.row - source.row).abs())
        <= radius
}

fn width_i32(size: GridSize) -> i32 {
    i32::try_from(size.width()).expect("grid width must fit in i32")
}

fn height_i32(size: GridSize) -> i32 {
    i32::try_from(size.height()).expect("grid height must fit in i32")
}

#[cfg(test)]
mod tests {
    use super::{
        BlockingRect, RectangleFov, perimeter_raycasting, perimeter_raycasting_into,
        rectangle_based_fov, recursive_shadowcasting,
    };
    use crate::{GameResult, Grid, GridSize, Point};

    fn grid_from_rows(rows: &[&str]) -> GameResult<Grid<char>> {
        let height = rows.len();
        let width = rows[0].chars().count();
        Grid::from_vec(
            GridSize::new(width, height)?,
            rows.iter().flat_map(|row| row.chars()).collect(),
        )
    }

    fn blocks_wall(point: Point, value: &char) -> bool {
        let _ = point;
        *value == '#'
    }

    fn visible_points(visible: &Grid<bool>) -> Vec<Point> {
        visible
            .iter()
            .filter_map(|(point, value)| (*value).then_some(point))
            .collect()
    }

    #[test]
    fn perimeter_raycasting_respects_radius() -> GameResult<()> {
        let map = Grid::new(GridSize::new(5, 5)?, '.')?;

        let visible = perimeter_raycasting(&map, Point::new(2, 2), Some(1), blocks_wall);

        assert_eq!(visible_points(&visible).len(), 9);
        assert!(visible[Point::new(1, 1)]);
        assert!(visible[Point::new(3, 3)]);
        assert!(!visible[Point::new(0, 2)]);
        Ok(())
    }

    #[test]
    fn perimeter_raycasting_stops_after_blocking_cell() -> GameResult<()> {
        let map = grid_from_rows(&["..#.."])?;

        let visible = perimeter_raycasting(&map, Point::new(0, 0), None, blocks_wall);

        assert!(visible[Point::new(0, 0)]);
        assert!(visible[Point::new(1, 0)]);
        assert!(visible[Point::new(2, 0)]);
        assert!(!visible[Point::new(3, 0)]);
        assert!(!visible[Point::new(4, 0)]);
        Ok(())
    }

    #[test]
    fn perimeter_raycasting_into_reuses_visibility_grid() -> GameResult<()> {
        let map = Grid::new(GridSize::new(3, 3)?, '.')?;
        let mut visible = Grid::new(map.size(), true)?;

        perimeter_raycasting_into(&map, &mut visible, Point::new(-1, 1), None, blocks_wall);

        assert!(visible.iter().all(|(_, value)| !*value));
        Ok(())
    }

    #[test]
    fn recursive_shadowcasting_sees_open_area_within_radius() -> GameResult<()> {
        let map = Grid::new(GridSize::new(5, 5)?, '.')?;

        let visible = recursive_shadowcasting(&map, Point::new(2, 2), Some(2), blocks_wall);

        assert_eq!(visible_points(&visible).len(), 25);
        assert!(visible[Point::new(0, 0)]);
        assert!(visible[Point::new(4, 4)]);
        Ok(())
    }

    #[test]
    fn recursive_shadowcasting_keeps_shadow_behind_wall() -> GameResult<()> {
        let map = grid_from_rows(&[".....", "...#.", ".S.#.", "...#.", "....."])?;

        let visible = recursive_shadowcasting(&map, Point::new(1, 2), None, blocks_wall);

        assert!(visible[Point::new(3, 2)]);
        assert!(!visible[Point::new(4, 2)]);
        Ok(())
    }

    #[test]
    fn rectangle_fov_groups_contiguous_blockers() -> GameResult<()> {
        let map = grid_from_rows(&[".....", ".##..", ".##..", "....."])?;

        let fov = RectangleFov::new(&map, blocks_wall);

        assert_eq!(
            fov.blocking_rectangles(),
            &[BlockingRect {
                min: Point::new(1, 1),
                max: Point::new(2, 2)
            }]
        );
        assert_eq!(fov.blocking_rectangles()[0].width(), 2);
        assert_eq!(fov.blocking_rectangles()[0].height(), 2);
        Ok(())
    }

    #[test]
    fn rectangle_based_fov_marks_shadowed_cells() -> GameResult<()> {
        let map = grid_from_rows(&[".......", "...#...", ".S.#...", "...#...", "......."])?;

        let visible = rectangle_based_fov(&map, Point::new(1, 2), None, blocks_wall);

        assert!(visible[Point::new(3, 2)]);
        assert!(!visible[Point::new(5, 2)]);
        assert!(visible[Point::new(1, 2)]);
        Ok(())
    }

    #[test]
    fn rectangle_based_fov_respects_radius() -> GameResult<()> {
        let map = Grid::new(GridSize::new(5, 5)?, '.')?;
        let fov = RectangleFov::new(&map, blocks_wall);

        let visible = fov.visible_from(Point::new(2, 2), Some(1));

        assert_eq!(visible_points(&visible).len(), 9);
        assert!(visible[Point::new(1, 2)]);
        assert!(!visible[Point::new(0, 2)]);
        Ok(())
    }
}
