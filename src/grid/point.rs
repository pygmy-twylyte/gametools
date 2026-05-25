//! # `Point`
//!
//! A location on a [`Grid`](crate::Grid), expressed as a column and row.
//!
//! # Examples
//!
//! ```
//! use gametools::{Point, PointDelta};
//!
//! let point = Point::new(2, 3);
//! assert_eq!(point + PointDelta::NORTH, Point::new(2, 2));
//! assert_eq!(Point::new(5, 1) - point, PointDelta::new(3, -2));
//! ```

use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::PointDelta;

/// A grid location.
///
/// `col` is the horizontal coordinate and `row` is the vertical coordinate.
/// Grid APIs treat `(0, 0)` as the top-left cell.
///
/// # Examples
///
/// ```
/// use gametools::Point;
///
/// let point = Point::new(4, 2);
/// assert_eq!(point.col, 4);
/// assert_eq!(point.row, 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct Point {
    /// Horizontal grid coordinate.
    pub col: i32,
    /// Vertical grid coordinate.
    pub row: i32,
}

impl Point {
    /// Creates a point from a column and row.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::Point;
    ///
    /// assert_eq!(Point::new(1, -2), Point { col: 1, row: -2 });
    /// ```
    pub const fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    /// Returns whether the point is in the same row as another point.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::Point;
    ///
    /// let point = Point::new(1, -2);
    /// assert!(point.same_row(Point::new(3, -2)));
    /// ```
    pub fn same_row(&self, other: Point) -> bool {
        self.row == other.row
    }

    /// Returns whether the point is in the same column as another point.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::Point;
    ///
    /// let point = Point::new(1, -2);
    /// assert!(point.same_col(Point::new(1, 3)));
    /// ```
    pub fn same_col(&self, other: Point) -> bool {
        self.col == other.col
    }

    /// Returns whether the point is on the same diagonal as another point.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::Point;
    ///
    /// let point = Point::new(1, -2);
    /// assert!(point.same_diagonal(Point::new(3, 1)));
    /// ```
    pub fn same_diagonal(&self, other: Point) -> bool {
        let delta = *self - other;
        delta.dc.abs() == delta.dr.abs()
    }
}

impl Add<PointDelta> for Point {
    type Output = Self;

    fn add(self, rhs: PointDelta) -> Self::Output {
        Self::new(self.col + rhs.dc, self.row + rhs.dr)
    }
}

impl AddAssign<PointDelta> for Point {
    fn add_assign(&mut self, rhs: PointDelta) {
        *self = *self + rhs;
    }
}

impl Sub<PointDelta> for Point {
    type Output = Self;

    fn sub(self, rhs: PointDelta) -> Self::Output {
        Self::new(self.col - rhs.dc, self.row - rhs.dr)
    }
}

impl SubAssign<PointDelta> for Point {
    fn sub_assign(&mut self, rhs: PointDelta) {
        *self = *self - rhs;
    }
}

impl Sub<Point> for Point {
    type Output = PointDelta;

    fn sub(self, rhs: Point) -> Self::Output {
        PointDelta::new(self.col - rhs.col, self.row - rhs.row)
    }
}

#[cfg(test)]
mod tests {
    use super::Point;
    use crate::PointDelta;

    #[test]
    fn new_and_default_create_expected_points() {
        assert_eq!(Point::new(3, -2), Point { col: 3, row: -2 });
        assert_eq!(Point::default(), Point::new(0, 0));
    }

    #[test]
    fn point_adds_and_subtracts_deltas() {
        let point = Point::new(4, 6);
        let delta = PointDelta::new(-2, 3);

        assert_eq!(point + delta, Point::new(2, 9));
        assert_eq!(point - delta, Point::new(6, 3));
    }

    #[test]
    fn point_add_assign_and_sub_assign_apply_deltas() {
        let mut point = Point::new(1, 1);

        point += PointDelta::EAST;
        point += PointDelta::SOUTH;
        assert_eq!(point, Point::new(2, 2));

        point -= PointDelta::NORTH_WEST;
        assert_eq!(point, Point::new(3, 3));
    }

    #[test]
    fn subtracting_points_returns_delta_between_them() {
        assert_eq!(Point::new(7, 2) - Point::new(3, 5), PointDelta::new(4, -3));
    }
}
