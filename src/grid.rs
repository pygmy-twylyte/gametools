//! # `Grid<T>`
//!
//! A rectangular, row-major grid for game boards, maps, pathfinding fields, and
//! other two-dimensional data.
//!
//! Cells are addressed with [`Point`] values rather than raw row/column pairs.
//! Points use `col` for the horizontal coordinate and `row` for the vertical
//! coordinate, with `(0, 0)` at the top-left corner.
//!
//! # Examples
//!
//! ```
//! use gametools::{GameResult, Grid, GridSize, Point};
//!
//! # fn main() -> GameResult<()> {
//! let size = GridSize::new(3, 2)?;
//! let grid = Grid::new_with_fn(size, |point| point.row * 10 + point.col)?;
//!
//! assert_eq!(grid[Point::new(0, 0)], 0);
//! assert_eq!(grid[Point::new(2, 1)], 12);
//! assert_eq!(grid.get(Point::new(3, 0)), None);
//! # Ok(()) }
//! ```

pub mod point;
pub use point::Point;

pub mod pointdelta;
pub use pointdelta::PointDelta;

use std::ops::{Index, IndexMut};

use crate::GameResult;
use crate::GridError;
use crate::ensure;

/// Non-zero dimensions used to create a [`Grid`].
///
/// `GridSize` validates that both dimensions are positive and that their area
/// fits in `usize`, so a constructed [`Grid`] can rely on those invariants.
///
/// # Examples
///
/// ```
/// use gametools::{GameResult, GridSize};
///
/// # fn main() -> GameResult<()> {
/// let size = GridSize::new(4, 3)?;
/// assert_eq!(size.width(), 4);
/// assert_eq!(size.height(), 3);
/// assert_eq!(size.area()?, 12);
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    width: usize,
    height: usize,
}

impl GridSize {
    /// Creates a validated grid size.
    ///
    /// # Errors
    /// - [`GridError::InvalidSize`] if either dimension is zero.
    /// - [`GridError::AreaOverflow`] if `width * height` overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameError, GridError, GridSize};
    ///
    /// let size = GridSize::new(2, 5).expect("valid size");
    /// assert_eq!(size.area().unwrap(), 10);
    ///
    /// assert_eq!(
    ///     GridSize::new(0, 5),
    ///     Err(GameError::GridError(GridError::InvalidSize(0, 5)))
    /// );
    /// ```
    pub fn new(width: usize, height: usize) -> GameResult<Self> {
        ensure!(
            width > 0 && height > 0,
            GridError::InvalidSize(width, height)
        );
        ensure!(width.checked_mul(height).is_some(), GridError::AreaOverflow);
        Ok(Self { width, height })
    }

    /// Returns the number of columns in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::GridSize;
    ///
    /// let size = GridSize::new(8, 6).unwrap();
    /// assert_eq!(size.width(), 8);
    /// ```
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the number of rows in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::GridSize;
    ///
    /// let size = GridSize::new(8, 6).unwrap();
    /// assert_eq!(size.height(), 6);
    /// ```
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the total number of cells represented by this size.
    ///
    /// # Errors
    /// Returns [`GridError::AreaOverflow`] if the dimensions overflow when
    /// multiplied. Values created with [`GridSize::new`] have already passed
    /// this check.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, GridSize};
    ///
    /// # fn main() -> GameResult<()> {
    /// let size = GridSize::new(3, 4)?;
    /// assert_eq!(size.area()?, 12);
    /// # Ok(()) }
    /// ```
    pub fn area(&self) -> GameResult<usize> {
        self.width
            .checked_mul(self.height)
            .ok_or(GridError::AreaOverflow.into())
    }
}

/// A generic rectangular grid addressed by [`Point`].
///
/// Values are stored in row-major order: all cells in row 0, then all cells in
/// row 1, and so on. This matters for [`Grid::from_vec`], [`Grid::points`],
/// [`Grid::iter`], and [`Grid::iter_mut`].
#[derive(Debug, Clone, PartialEq)]
pub struct Grid<T> {
    cells: Vec<T>,
    size: GridSize,
}

impl<T: Clone> Grid<T> {
    /// Creates a grid where every cell is initialized with `filler`.
    ///
    /// # Errors
    /// Returns an error if the area of the grid exceeds `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::new(GridSize::new(2, 2)?, false)?;
    ///
    /// assert_eq!(grid[Point::new(0, 0)], false);
    /// assert_eq!(grid[Point::new(1, 1)], false);
    /// # Ok(()) }
    /// ```
    pub fn new(size: GridSize, filler: T) -> GameResult<Self> {
        Ok(Self {
            cells: vec![filler; size.area()?],
            size,
        })
    }
}

impl<T> Grid<T> {
    /// Creates a grid from row-major cell data.
    ///
    /// # Errors
    /// Returns an error if the vector length does not match `size.area()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::from_vec(GridSize::new(3, 2)?, vec![1, 2, 3, 4, 5, 6])?;
    ///
    /// assert_eq!(grid[Point::new(0, 0)], 1);
    /// assert_eq!(grid[Point::new(2, 0)], 3);
    /// assert_eq!(grid[Point::new(0, 1)], 4);
    /// # Ok(()) }
    /// ```
    pub fn from_vec(size: GridSize, cells: Vec<T>) -> GameResult<Self> {
        let area = size.area()?;
        ensure!(
            cells.len() == area,
            GridError::CellCountMismatch {
                actual: cells.len(),
                expected: area
            }
        );
        Ok(Self { cells, size })
    }

    /// Creates a grid by calling `filler` once for each point.
    ///
    /// Points are supplied in row-major order.
    ///
    /// # Errors
    /// Returns an error if the area of the grid exceeds `usize::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::new_with_fn(GridSize::new(3, 2)?, |point| {
    ///     point.row * 10 + point.col
    /// })?;
    ///
    /// assert_eq!(grid[Point::new(2, 1)], 12);
    /// # Ok(()) }
    /// ```
    pub fn new_with_fn<F>(size: GridSize, mut filler: F) -> GameResult<Self>
    where
        F: FnMut(Point) -> T,
    {
        let mut cells: Vec<T> = Vec::with_capacity(size.area()?);
        for row in 0..size.height {
            for col in 0..size.width {
                cells.push(filler(Point {
                    col: i32::try_from(col).map_err(|_| GridError::DimensionTruncated("width"))?,
                    row: i32::try_from(row).map_err(|_| GridError::DimensionTruncated("height"))?,
                }));
            }
        }
        Ok(Self { cells, size })
    }

    /// Returns the dimensions of the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize};
    ///
    /// # fn main() -> GameResult<()> {
    /// let size = GridSize::new(5, 4)?;
    /// let grid = Grid::new(size, 0)?;
    ///
    /// assert_eq!(grid.size(), size);
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn size(&self) -> GridSize {
        self.size
    }

    /// Returns a reference to the cell at the specified point, if it is within the grid's bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::from_vec(GridSize::new(2, 1)?, vec!["left", "right"])?;
    ///
    /// assert_eq!(grid.get(Point::new(1, 0)), Some(&"right"));
    /// assert_eq!(grid.get(Point::new(2, 0)), None);
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn get(&self, cell: Point) -> Option<&T> {
        self.point_to_index(cell).map(|index| &self.cells[index])
    }

    /// Returns a mutable reference to the cell at the specified point, if it is within the grid's bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut grid = Grid::new(GridSize::new(2, 1)?, 0)?;
    /// *grid.get_mut(Point::new(1, 0)).unwrap() = 7;
    ///
    /// assert_eq!(grid[Point::new(1, 0)], 7);
    /// assert!(grid.get_mut(Point::new(-1, 0)).is_none());
    /// # Ok(()) }
    /// ```
    pub fn get_mut(&mut self, cell: Point) -> Option<&mut T> {
        self.point_to_index(cell)
            .map(|index| &mut self.cells[index])
    }

    /// Returns all valid grid points in row-major order.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::new(GridSize::new(2, 2)?, ())?;
    ///
    /// assert_eq!(
    ///     grid.points().collect::<Vec<_>>(),
    ///     vec![
    ///         Point::new(0, 0),
    ///         Point::new(1, 0),
    ///         Point::new(0, 1),
    ///         Point::new(1, 1),
    ///     ]
    /// );
    /// # Ok(()) }
    /// ```
    pub fn points(&self) -> impl Iterator<Item = Point> {
        let width = self.size.width;
        (0..self.cells.len()).map(move |index| index_to_point(index, width))
    }

    /// Returns an iterator over all the cells in the grid, with their corresponding points.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::from_vec(GridSize::new(2, 1)?, vec!["a", "b"])?;
    /// let cells = grid.iter().map(|(point, value)| (point, *value)).collect::<Vec<_>>();
    ///
    /// assert_eq!(cells, vec![(Point::new(0, 0), "a"), (Point::new(1, 0), "b")]);
    /// # Ok(()) }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (Point, &T)> {
        let width = self.size.width;
        self.cells
            .iter()
            .enumerate()
            .map(move |(idx, cell)| (index_to_point(idx, width), cell))
    }

    /// Returns an iterator over all points and mutable cell references.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut grid = Grid::new(GridSize::new(2, 2)?, 0)?;
    /// for (point, value) in grid.iter_mut() {
    ///     *value = point.col + point.row;
    /// }
    ///
    /// assert_eq!(grid[Point::new(1, 1)], 2);
    /// # Ok(()) }
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Point, &mut T)> {
        let width = self.size.width;
        self.cells
            .iter_mut()
            .enumerate()
            .map(move |(idx, cell)| (index_to_point(idx, width), cell))
    }

    /// Returns `true` if the point is within the grid's bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::new(GridSize::new(2, 2)?, ())?;
    ///
    /// assert!(grid.is_in_bounds(Point::new(1, 1)));
    /// assert!(!grid.is_in_bounds(Point::new(2, 1)));
    /// assert!(!grid.is_in_bounds(Point::new(-1, 0)));
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn is_in_bounds(&self, point: Point) -> bool {
        self.point_to_index(point).is_some()
    }

    /// Alias for [`Grid::is_in_bounds()`], returns true if the grid contains the point.
    #[must_use]
    pub fn contains_point(&self, point: Point) -> bool {
        self.is_in_bounds(point)
    }

    /// Converts a point to an index, returning `None` if the point is out of bounds.
    fn point_to_index(&self, point: Point) -> Option<usize> {
        let row = usize::try_from(point.row).ok()?;
        let col = usize::try_from(point.col).ok()?;
        if row >= self.size.height || col >= self.size.width {
            return None;
        }

        let index = self.size.width * row + col;
        if index >= self.cells.len() {
            return None;
        }

        Some(index)
    }

    fn neighbors_inner(
        &self,
        center: Point,
        deltas: &[PointDelta],
    ) -> impl Iterator<Item = (Point, &T)> {
        deltas.iter().filter_map(move |delta| {
            let point = center + *delta;
            self.get(point).map(|value| (point, value))
        })
    }

    fn neighbors_inner_mut(
        &mut self,
        center: Point,
        deltas: &[PointDelta],
    ) -> impl Iterator<Item = (Point, &mut T)> {
        let neighbors: Vec<(Point, usize)> = deltas
            .iter()
            .filter_map(|delta| {
                let point = center + *delta;
                self.point_to_index(point).map(|index| (point, index))
            })
            .collect();

        self.cells
            .iter_mut()
            .enumerate()
            .filter_map(move |(index, value)| {
                neighbors
                    .iter()
                    .find_map(|(point, neighbor_index)| {
                        (*neighbor_index == index).then_some(*point)
                    })
                    .map(|point| (point, value))
            })
    }

    /// Returns the in-bounds north, south, east, and west neighbors of `center`.
    ///
    /// Neighbors are returned in [`PointDelta::CARDINALS`] order with
    /// out-of-bounds points skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::from_vec(GridSize::new(3, 3)?, (0..9).collect())?;
    /// let neighbors = grid
    ///     .cardinal_neighbors(Point::new(1, 1))
    ///     .map(|(point, value)| (point, *value))
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     neighbors,
    ///     vec![
    ///         (Point::new(1, 0), 1),
    ///         (Point::new(0, 1), 3),
    ///         (Point::new(2, 1), 5),
    ///         (Point::new(1, 2), 7),
    ///     ]
    /// );
    /// # Ok(()) }
    /// ```
    pub fn cardinal_neighbors(&self, center: Point) -> impl Iterator<Item = (Point, &T)> {
        self.neighbors_inner(center, &PointDelta::CARDINALS)
    }

    /// Returns mutable references to the in-bounds cardinal neighbors of `center`.
    ///
    /// Mutable neighbors are yielded in backing storage order, not delta order.
    /// This keeps the implementation safe while still guaranteeing each yielded
    /// `&mut T` refers to a distinct cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut grid = Grid::new(GridSize::new(3, 3)?, 0)?;
    /// for (_, value) in grid.cardinal_neighbors_mut(Point::new(1, 1)) {
    ///     *value = 1;
    /// }
    ///
    /// assert_eq!(grid[Point::new(1, 0)], 1);
    /// assert_eq!(grid[Point::new(1, 1)], 0);
    /// assert_eq!(grid[Point::new(1, 2)], 1);
    /// # Ok(()) }
    /// ```
    pub fn cardinal_neighbors_mut(
        &mut self,
        center: Point,
    ) -> impl Iterator<Item = (Point, &mut T)> {
        self.neighbors_inner_mut(center, &PointDelta::CARDINALS)
    }

    /// Returns the in-bounds diagonal neighbors of `center`.
    ///
    /// Neighbors are returned in [`PointDelta::DIAGONALS`] order with
    /// out-of-bounds points skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::from_vec(GridSize::new(3, 3)?, (0..9).collect())?;
    /// let neighbors = grid
    ///     .diagonal_neighbors(Point::new(0, 0))
    ///     .map(|(point, value)| (point, *value))
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(neighbors, vec![(Point::new(1, 1), 4)]);
    /// # Ok(()) }
    /// ```
    pub fn diagonal_neighbors(&self, center: Point) -> impl Iterator<Item = (Point, &T)> {
        self.neighbors_inner(center, &PointDelta::DIAGONALS)
    }

    /// Returns mutable references to the in-bounds diagonal neighbors of `center`.
    ///
    /// Mutable neighbors are yielded in backing storage order, not delta order.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut grid = Grid::new(GridSize::new(3, 3)?, 0)?;
    /// for (_, value) in grid.diagonal_neighbors_mut(Point::new(1, 1)) {
    ///     *value = 2;
    /// }
    ///
    /// assert_eq!(grid[Point::new(0, 0)], 2);
    /// assert_eq!(grid[Point::new(1, 1)], 0);
    /// assert_eq!(grid[Point::new(2, 2)], 2);
    /// # Ok(()) }
    /// ```
    pub fn diagonal_neighbors_mut(
        &mut self,
        center: Point,
    ) -> impl Iterator<Item = (Point, &mut T)> {
        self.neighbors_inner_mut(center, &PointDelta::DIAGONALS)
    }

    /// Returns all in-bounds cardinal and diagonal neighbors of `center`.
    ///
    /// Neighbors are returned in [`PointDelta::ALL_DIRECTIONS`] order with
    /// out-of-bounds points skipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let grid = Grid::new(GridSize::new(3, 3)?, '.')?;
    ///
    /// assert_eq!(grid.all_neighbors(Point::new(1, 1)).count(), 8);
    /// assert_eq!(grid.all_neighbors(Point::new(0, 0)).count(), 3);
    /// # Ok(()) }
    /// ```
    pub fn all_neighbors(&self, center: Point) -> impl Iterator<Item = (Point, &T)> {
        self.neighbors_inner(center, &PointDelta::ALL_DIRECTIONS)
    }

    /// Returns mutable references to all in-bounds neighbors of `center`.
    ///
    /// Mutable neighbors are yielded in backing storage order, not delta order.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::{GameResult, Grid, GridSize, Point};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut grid = Grid::new(GridSize::new(3, 3)?, 0)?;
    /// for (_, value) in grid.all_neighbors_mut(Point::new(0, 0)) {
    ///     *value += 1;
    /// }
    ///
    /// assert_eq!(grid[Point::new(0, 0)], 0);
    /// assert_eq!(grid[Point::new(1, 0)], 1);
    /// assert_eq!(grid[Point::new(0, 1)], 1);
    /// assert_eq!(grid[Point::new(1, 1)], 1);
    /// # Ok(()) }
    /// ```
    pub fn all_neighbors_mut(&mut self, center: Point) -> impl Iterator<Item = (Point, &mut T)> {
        self.neighbors_inner_mut(center, &PointDelta::ALL_DIRECTIONS)
    }

    /// Returns an iterator over the cells in the row of the given point.
    pub fn row_at(&self, cell: Point) -> impl Iterator<Item = (Point, &T)> {
        let desired = cell.row;
        let width = self.size.width;
        self.cells
            .iter()
            .enumerate()
            .filter_map(move |(idx, cell)| {
                let point = index_to_point(idx, width);
                if point.row == desired {
                    Some((point, cell))
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over the cells in the row of the given point.
    pub fn row_at_mut(&mut self, cell: Point) -> impl Iterator<Item = (Point, &mut T)> {
        let desired = cell.row;
        let width = self.size.width;
        self.cells
            .iter_mut()
            .enumerate()
            .filter_map(move |(idx, cell)| {
                let point = index_to_point(idx, width);
                if point.row == desired {
                    Some((point, cell))
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over the cells of the column containing the given point.
    pub fn col_at(&self, cell: Point) -> impl Iterator<Item = (Point, &T)> {
        let desired = cell.col;
        let width = self.size.width;
        self.cells
            .iter()
            .enumerate()
            .filter_map(move |(idx, cell)| {
                let point = index_to_point(idx, width);
                if point.col == desired {
                    Some((point, cell))
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over the cells of the column containing the given point with
    /// mutable access to each cell.
    pub fn col_at_mut(&mut self, cell: Point) -> impl Iterator<Item = (Point, &mut T)> {
        let desired = cell.col;
        let width = self.size.width;
        self.cells
            .iter_mut()
            .enumerate()
            .filter_map(move |(idx, cell)| {
                let point = index_to_point(idx, width);
                if point.col == desired {
                    Some((point, cell))
                } else {
                    None
                }
            })
    }
}

impl<T> Index<Point> for Grid<T> {
    type Output = T;
    fn index(&self, point: Point) -> &Self::Output {
        &self.cells[self.point_to_index(point).unwrap()]
    }
}

impl<T> IndexMut<Point> for Grid<T> {
    fn index_mut(&mut self, point: Point) -> &mut Self::Output {
        let idx = self
            .point_to_index(point)
            .expect("point index out of bounds");
        &mut self.cells[idx]
    }
}

/// Convert a vector index value to a Point, given the width of a grid.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
fn index_to_point(index: usize, width: usize) -> Point {
    Point {
        col: (index % width) as i32,
        row: (index / width) as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::{Grid, GridSize, Point};
    use crate::{GameError, GridError};
    use std::collections::BTreeMap;
    use std::ops::IndexMut;

    fn size(width: usize, height: usize) -> GridSize {
        GridSize::new(width, height).expect("valid grid size")
    }

    fn sample_grid() -> Grid<i32> {
        Grid::from_vec(size(3, 3), (0..9).collect()).expect("valid grid")
    }

    fn collect_neighbor_values<'a>(
        neighbors: impl Iterator<Item = (Point, &'a i32)>,
    ) -> Vec<(Point, i32)> {
        neighbors.map(|(point, value)| (point, *value)).collect()
    }

    #[test]
    fn grid_size_new_accepts_positive_dimensions() {
        let size = GridSize::new(4, 7).expect("positive dimensions are valid");

        assert_eq!(size.width(), 4);
        assert_eq!(size.height(), 7);
        assert_eq!(size.area().expect("area fits"), 28);
    }

    #[test]
    fn grid_size_new_rejects_zero_dimensions() {
        assert_eq!(
            GridSize::new(0, 2),
            Err(GameError::GridError(GridError::InvalidSize(0, 2)))
        );
        assert_eq!(
            GridSize::new(2, 0),
            Err(GameError::GridError(GridError::InvalidSize(2, 0)))
        );
    }

    #[test]
    fn grid_size_new_rejects_overflowing_area() {
        assert_eq!(
            GridSize::new(usize::MAX, 2),
            Err(GameError::GridError(GridError::AreaOverflow))
        );
    }

    #[test]
    fn new_fills_every_cell() {
        let grid = Grid::new(size(3, 2), "x").expect("valid grid");

        assert_eq!(grid.size(), size(3, 2));
        assert!(grid.iter().all(|(_, value)| *value == "x"));
        assert_eq!(grid.iter().count(), 6);
    }

    #[test]
    fn from_vec_preserves_row_major_cells() {
        let grid = Grid::from_vec(size(3, 2), vec![10, 11, 12, 20, 21, 22]).expect("valid grid");

        assert_eq!(grid[Point::new(0, 0)], 10);
        assert_eq!(grid[Point::new(2, 0)], 12);
        assert_eq!(grid[Point::new(0, 1)], 20);
        assert_eq!(grid[Point::new(2, 1)], 22);
    }

    #[test]
    fn from_vec_rejects_cell_count_mismatch() {
        assert_eq!(
            Grid::from_vec(size(2, 3), vec![1, 2, 3]),
            Err(GameError::GridError(GridError::CellCountMismatch {
                actual: 3,
                expected: 6
            }))
        );
    }

    #[test]
    fn new_with_fn_receives_points_in_row_major_order() {
        let mut visited = Vec::new();
        let grid = Grid::new_with_fn(size(3, 2), |point| {
            visited.push(point);
            point.row * 10 + point.col
        })
        .expect("valid grid");

        assert_eq!(
            visited,
            vec![
                Point::new(0, 0),
                Point::new(1, 0),
                Point::new(2, 0),
                Point::new(0, 1),
                Point::new(1, 1),
                Point::new(2, 1),
            ]
        );
        assert_eq!(grid[Point::new(2, 1)], 12);
    }

    #[test]
    fn get_and_bounds_check_reject_negative_and_oversized_points() {
        let grid = sample_grid();

        assert_eq!(grid.get(Point::new(1, 1)), Some(&4));
        assert!(grid.is_in_bounds(Point::new(2, 2)));
        assert!(grid.contains_point(Point::new(2, 2)));
        assert!(!grid.is_in_bounds(Point::new(-1, 0)));
        assert!(!grid.contains_point(Point::new(-1, 0)));
        assert!(!grid.is_in_bounds(Point::new(0, -1)));
        assert!(!grid.is_in_bounds(Point::new(3, 0)));
        assert!(!grid.is_in_bounds(Point::new(0, 3)));
        assert_eq!(grid.get(Point::new(4, 0)), None);
    }

    #[test]
    fn get_mut_updates_in_bounds_cells_only() {
        let mut grid = sample_grid();

        *grid.get_mut(Point::new(1, 1)).expect("cell exists") = 99;

        assert_eq!(grid.get(Point::new(1, 1)), Some(&99));
        assert_eq!(grid.get_mut(Point::new(3, 1)), None);
    }

    #[test]
    fn point_to_index_rejects_points_beyond_backing_storage() {
        let grid = Grid {
            cells: vec![1],
            size: size(2, 2),
        };

        assert_eq!(grid.point_to_index(Point::new(0, 0)), Some(0));
        assert_eq!(grid.point_to_index(Point::new(1, 0)), None);
    }

    #[test]
    fn points_iterates_in_row_major_order() {
        let grid = Grid::new(size(2, 2), ()).expect("valid grid");

        assert_eq!(
            grid.points().collect::<Vec<_>>(),
            vec![
                Point::new(0, 0),
                Point::new(1, 0),
                Point::new(0, 1),
                Point::new(1, 1),
            ]
        );
    }

    #[test]
    fn iter_pairs_points_with_values() {
        let grid = Grid::from_vec(size(2, 2), vec!['a', 'b', 'c', 'd']).expect("valid grid");

        assert_eq!(
            grid.iter()
                .map(|(point, value)| (point, *value))
                .collect::<Vec<_>>(),
            vec![
                (Point::new(0, 0), 'a'),
                (Point::new(1, 0), 'b'),
                (Point::new(0, 1), 'c'),
                (Point::new(1, 1), 'd'),
            ]
        );
    }

    #[test]
    fn iter_mut_pairs_points_with_mutable_values() {
        let mut grid =
            Grid::new_with_fn(size(2, 2), |point| point.row * 10 + point.col).expect("valid grid");

        for (point, value) in grid.iter_mut() {
            *value += point.col + point.row;
        }

        assert_eq!(grid[Point::new(0, 0)], 0);
        assert_eq!(grid[Point::new(1, 0)], 2);
        assert_eq!(grid[Point::new(0, 1)], 11);
        assert_eq!(grid[Point::new(1, 1)], 13);
    }

    #[test]
    fn index_mut_updates_cell() {
        let mut grid = sample_grid();

        grid[Point::new(2, 2)] = 42;
        *grid.index_mut(Point::new(0, 0)) = 24;

        assert_eq!(grid[Point::new(2, 2)], 42);
        assert_eq!(grid[Point::new(0, 0)], 24);
    }

    #[test]
    fn cardinal_neighbors_filters_to_in_bounds_cells() {
        let grid = sample_grid();

        assert_eq!(
            collect_neighbor_values(grid.cardinal_neighbors(Point::new(1, 1))),
            vec![
                (Point::new(1, 0), 1),
                (Point::new(0, 1), 3),
                (Point::new(2, 1), 5),
                (Point::new(1, 2), 7),
            ]
        );
        assert_eq!(
            collect_neighbor_values(grid.cardinal_neighbors(Point::new(0, 0))),
            vec![(Point::new(1, 0), 1), (Point::new(0, 1), 3),]
        );
    }

    #[test]
    fn diagonal_neighbors_filters_to_in_bounds_cells() {
        let grid = sample_grid();

        assert_eq!(
            collect_neighbor_values(grid.diagonal_neighbors(Point::new(1, 1))),
            vec![
                (Point::new(0, 0), 0),
                (Point::new(2, 0), 2),
                (Point::new(0, 2), 6),
                (Point::new(2, 2), 8),
            ]
        );
        assert_eq!(
            collect_neighbor_values(grid.diagonal_neighbors(Point::new(0, 0))),
            vec![(Point::new(1, 1), 4)]
        );
    }

    #[test]
    fn all_neighbors_filters_to_in_bounds_cells() {
        let grid = sample_grid();

        assert_eq!(grid.all_neighbors(Point::new(1, 1)).count(), 8);
        assert_eq!(
            collect_neighbor_values(grid.all_neighbors(Point::new(0, 0))),
            vec![
                (Point::new(1, 0), 1),
                (Point::new(0, 1), 3),
                (Point::new(1, 1), 4),
            ]
        );
    }

    #[test]
    fn mutable_neighbor_iterators_update_only_neighbors() {
        let mut grid = Grid::new(size(3, 3), 0).expect("valid grid");

        for (_, value) in grid.cardinal_neighbors_mut(Point::new(1, 1)) {
            *value += 1;
        }
        for (_, value) in grid.diagonal_neighbors_mut(Point::new(1, 1)) {
            *value += 10;
        }
        for (_, value) in grid.all_neighbors_mut(Point::new(0, 0)) {
            *value += 100;
        }

        let values = grid
            .iter()
            .map(|(point, value)| (point, *value))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(values[&Point::new(0, 0)], 10);
        assert_eq!(values[&Point::new(1, 0)], 101);
        assert_eq!(values[&Point::new(2, 0)], 10);
        assert_eq!(values[&Point::new(0, 1)], 101);
        assert_eq!(values[&Point::new(1, 1)], 100);
        assert_eq!(values[&Point::new(2, 1)], 1);
        assert_eq!(values[&Point::new(0, 2)], 10);
        assert_eq!(values[&Point::new(1, 2)], 1);
        assert_eq!(values[&Point::new(2, 2)], 10);
    }

    #[test]
    fn row_at_returns_matching_row_cells() {
        let grid = sample_grid();

        assert_eq!(
            collect_neighbor_values(grid.row_at(Point::new(1, 1))),
            vec![
                (Point::new(0, 1), 3),
                (Point::new(1, 1), 4),
                (Point::new(2, 1), 5),
            ]
        );
        assert!(grid.row_at(Point::new(1, -1)).next().is_none());
        assert!(grid.row_at(Point::new(1, 3)).next().is_none());
    }

    #[test]
    fn row_at_mut_updates_matching_row_cells() {
        let mut grid = sample_grid();

        for (point, value) in grid.row_at_mut(Point::new(99, 1)) {
            *value += point.col;
        }

        assert_eq!(grid[Point::new(0, 0)], 0);
        assert_eq!(grid[Point::new(0, 1)], 3);
        assert_eq!(grid[Point::new(1, 1)], 5);
        assert_eq!(grid[Point::new(2, 1)], 7);
        assert_eq!(grid[Point::new(2, 2)], 8);

        assert!(grid.row_at_mut(Point::new(0, -1)).next().is_none());
        assert!(grid.row_at_mut(Point::new(0, 3)).next().is_none());
    }

    #[test]
    fn col_at_returns_matching_column_cells() {
        let grid = sample_grid();

        assert_eq!(
            collect_neighbor_values(grid.col_at(Point::new(1, 99))),
            vec![
                (Point::new(1, 0), 1),
                (Point::new(1, 1), 4),
                (Point::new(1, 2), 7),
            ]
        );
        assert!(grid.col_at(Point::new(-1, 1)).next().is_none());
        assert!(grid.col_at(Point::new(3, 1)).next().is_none());
    }

    #[test]
    fn col_at_mut_updates_matching_column_cells() {
        let mut grid = sample_grid();

        for (point, value) in grid.col_at_mut(Point::new(1, 99)) {
            *value += point.row;
        }

        assert_eq!(grid[Point::new(0, 0)], 0);
        assert_eq!(grid[Point::new(1, 0)], 1);
        assert_eq!(grid[Point::new(1, 1)], 5);
        assert_eq!(grid[Point::new(1, 2)], 9);
        assert_eq!(grid[Point::new(2, 2)], 8);

        assert!(grid.col_at_mut(Point::new(-1, 0)).next().is_none());
        assert!(grid.col_at_mut(Point::new(3, 0)).next().is_none());
    }
}
