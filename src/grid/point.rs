//! # `Point`

use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::PointDelta;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct Point {
    pub col: i32,
    pub row: i32,
}

impl Point {
    pub const fn new(col: i32, row: i32) -> Self {
        Self { col, row }
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
