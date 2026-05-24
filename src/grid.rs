//! 'Grid<T>`
//!
//! A generic grid data structure.

pub mod point;

use std::ops::{Deref, Index, IndexMut};

use crate::grid::point::Point;

/// A wrapper around `usize` representing the width of a grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Width(usize);
impl Deref for Width {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A wrapper around `usize` representing the height of a grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Height(usize);
impl Deref for Height {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A generic grid data structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid<T> {
    cells: Vec<T>,
    cols: Width,
    rows: Height,
}

impl<T: Clone> Grid<T> {
    /// Creates a new grid with the specified width, height, and filler value.
    pub fn new(cols: Width, rows: Height, filler: &T) -> Self {
        Self {
            cells: vec![filler.clone(); *cols * *rows],
            cols,
            rows,
        }
    }

    /// Creates a new grid, filling the cells using a supplied function to determine initial values.
    pub fn new_with_fn<F>(cols: Width, rows: Height, mut filler: F) -> Self
    where
        F: FnMut(Point) -> T,
    {
        let mut cells: Vec<T> = Vec::with_capacity(*cols * *rows);
        for row in 0..*rows {
            for col in 0..*cols {
                cells.push(filler(Point {
                    col: col as i32,
                    row: row as i32,
                }));
            }
        }
        Self { cells, cols, rows }
    }

    /// Returns a reference to the cell at the specified point, if it is within the grid's bounds.
    pub fn get(&self, cell: Point) -> Option<&T> {
        self.point_to_index(cell).map(|index| &self.cells[index])
    }

    /// Returns a mutable reference to the cell at the specified point, if it is within the grid's bounds.
    pub fn get_mut(&mut self, cell: Point) -> Option<&mut T> {
        self.point_to_index(cell)
            .map(|index| &mut self.cells[index])
    }
}

impl<T> Grid<T> {
    /// Returns `true` if the point is within the grid's bounds.
    pub fn is_in_bounds(&self, point: Point) -> bool {
        self.point_to_index(point).is_some()
    }

    /// Converts an index to a point, returning `None` if the index is out of bounds.
    fn index_to_point(&self, index: usize) -> Option<Point> {
        if index >= self.cells.len() {
            return None;
        }
        let row = index / *self.cols;
        let col = index % *self.cols;
        Some(Point {
            col: i32::try_from(col).ok()?,
            row: i32::try_from(row).ok()?,
        })
    }

    /// Converts a point to an index, returning `None` if the point is out of bounds.
    fn point_to_index(&self, point: Point) -> Option<usize> {
        let row = usize::try_from(point.row).ok()?;
        let col = usize::try_from(point.col).ok()?;
        let index = *self.cols * row + col;
        if index >= self.cells.len() {
            return None;
        }
        Some(index)
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
