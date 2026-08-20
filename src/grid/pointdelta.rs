//! # `PointDelta`
//!
//! Directional offsets that can be added to or subtracted from a
//! [`Point`](crate::Point).
//!
//! # Examples
//!
//! ```
//! use gametools::{Point, PointDelta};
//!
//! let start = Point::new(3, 3);
//! assert_eq!(start + PointDelta::NORTH_EAST, Point::new(4, 2));
//! assert_eq!(PointDelta::SOUTH.inverted(), PointDelta::NORTH);
//! ```

use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

/// A change in grid position.
///
/// `dc` is the column delta and `dr` is the row delta. Negative rows move north
/// in the default grid coordinate system.
///
/// # Examples
///
/// ```
/// use gametools::{Point, PointDelta};
///
/// let delta = PointDelta::new(-1, 2);
/// assert_eq!(Point::new(5, 5) + delta, Point::new(4, 7));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct PointDelta {
    /// Change in column.
    pub dc: i32,
    /// Change in row.
    pub dr: i32,
}

macro_rules! delta {
    ($dc:expr, $dr:expr) => {
        PointDelta { dc: $dc, dr: $dr }
    };
}

impl PointDelta {
    /// Creates a point delta from a column delta and row delta.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// assert_eq!(PointDelta::new(2, -1), PointDelta { dc: 2, dr: -1 });
    /// ```
    #[must_use]
    pub const fn new(dc: i32, dr: i32) -> Self {
        Self { dc, dr }
    }

    /// Returns the opposite delta.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// assert_eq!(PointDelta::NORTH.inverted(), PointDelta::SOUTH);
    /// assert_eq!(PointDelta::new(2, -3).inverted(), PointDelta::new(-2, 3));
    /// ```
    #[must_use]
    pub const fn inverted(self) -> Self {
        Self::new(-self.dc, -self.dr)
    }

    /// Returns the Euclidean distance covered by a delta.
    ///
    /// Uses the Pythagorean theorem to calculate the distance.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// let delta = PointDelta::new(3, 4);
    /// assert_eq!(delta.distance_euclid(), 5.0);
    /// ```
    #[must_use]
    pub fn distance_euclid(&self) -> f64 {
        f64::from(self.dc).hypot(f64::from(self.dr))
    }

    /// Returns the taxicab distance covered by a delta.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// let delta = PointDelta::new(3, 4);
    /// assert_eq!(delta.distance_taxicab(), 7);
    /// ```
    #[must_use]
    pub fn distance_taxicab(&self) -> u32 {
        self.dc.abs().saturating_add(self.dr.abs()).cast_unsigned()
    }

    /// Returns the Chebyshev distance covered by a delta.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// let delta = PointDelta::new(3, 4);
    /// assert_eq!(delta.distance_chebyshev(), 4);
    /// ```
    #[must_use]
    pub fn distance_chebyshev(&self) -> u32 {
        self.dc.abs().max(self.dr.abs()).cast_unsigned()
    }

    /// One row up.
    pub const NORTH: Self = delta!(0, -1);
    /// One row down.
    pub const SOUTH: Self = delta!(0, 1);
    /// One column right.
    pub const EAST: Self = delta!(1, 0);
    /// One column left.
    pub const WEST: Self = delta!(-1, 0);
    /// One column right and one row up.
    pub const NORTH_EAST: Self = delta!(1, -1);
    /// One column left and one row up.
    pub const NORTH_WEST: Self = delta!(-1, -1);
    /// One column right and one row down.
    pub const SOUTH_EAST: Self = delta!(1, 1);
    /// One column left and one row down.
    pub const SOUTH_WEST: Self = delta!(-1, 1);

    /// The four diagonal directions.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// assert_eq!(PointDelta::DIAGONALS.len(), 4);
    /// assert!(PointDelta::DIAGONALS.contains(&PointDelta::NORTH_EAST));
    /// ```
    pub const DIAGONALS: [Self; 4] = [
        Self::NORTH_WEST,
        Self::NORTH_EAST,
        Self::SOUTH_WEST,
        Self::SOUTH_EAST,
    ];

    /// The four orthogonal directions.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// assert_eq!(PointDelta::CARDINALS.len(), 4);
    /// assert!(PointDelta::CARDINALS.contains(&PointDelta::WEST));
    /// ```
    pub const CARDINALS: [Self; 4] = [Self::NORTH, Self::WEST, Self::EAST, Self::SOUTH];

    /// All eight surrounding directions.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::PointDelta;
    ///
    /// assert_eq!(PointDelta::ALL_DIRECTIONS.len(), 8);
    /// assert!(PointDelta::ALL_DIRECTIONS.contains(&PointDelta::SOUTH_WEST));
    /// ```
    pub const ALL_DIRECTIONS: [Self; 8] = [
        Self::NORTH_WEST,
        Self::NORTH,
        Self::NORTH_EAST,
        Self::WEST,
        Self::EAST,
        Self::SOUTH_WEST,
        Self::SOUTH,
        Self::SOUTH_EAST,
    ];
}

impl Add<PointDelta> for PointDelta {
    type Output = PointDelta;

    fn add(self, rhs: PointDelta) -> Self::Output {
        PointDelta::new(self.dc + rhs.dc, self.dr + rhs.dr)
    }
}

impl Sub<PointDelta> for PointDelta {
    type Output = PointDelta;

    fn sub(self, rhs: PointDelta) -> Self::Output {
        PointDelta::new(self.dc - rhs.dc, self.dr - rhs.dr)
    }
}

impl AddAssign<PointDelta> for PointDelta {
    fn add_assign(&mut self, rhs: PointDelta) {
        *self = *self + rhs;
    }
}

impl SubAssign<PointDelta> for PointDelta {
    fn sub_assign(&mut self, rhs: PointDelta) {
        *self = *self - rhs;
    }
}

impl Mul<u32> for PointDelta {
    type Output = PointDelta;

    fn mul(self, rhs: u32) -> Self::Output {
        PointDelta::new(self.dc * rhs.cast_signed(), self.dr * rhs.cast_signed())
    }
}

impl MulAssign<u32> for PointDelta {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::PointDelta;

    #[test]
    fn new_default_and_direction_constants_have_expected_values() {
        assert_eq!(PointDelta::new(3, -4), PointDelta { dc: 3, dr: -4 });
        assert_eq!(PointDelta::default(), PointDelta::new(0, 0));
        assert_eq!(PointDelta::NORTH, PointDelta::new(0, -1));
        assert_eq!(PointDelta::SOUTH, PointDelta::new(0, 1));
        assert_eq!(PointDelta::EAST, PointDelta::new(1, 0));
        assert_eq!(PointDelta::WEST, PointDelta::new(-1, 0));
        assert_eq!(PointDelta::NORTH_EAST, PointDelta::new(1, -1));
        assert_eq!(PointDelta::NORTH_WEST, PointDelta::new(-1, -1));
        assert_eq!(PointDelta::SOUTH_EAST, PointDelta::new(1, 1));
        assert_eq!(PointDelta::SOUTH_WEST, PointDelta::new(-1, 1));
    }

    #[test]
    fn direction_groups_have_expected_order() {
        assert_eq!(
            PointDelta::CARDINALS,
            [
                PointDelta::NORTH,
                PointDelta::WEST,
                PointDelta::EAST,
                PointDelta::SOUTH,
            ]
        );
        assert_eq!(
            PointDelta::DIAGONALS,
            [
                PointDelta::NORTH_WEST,
                PointDelta::NORTH_EAST,
                PointDelta::SOUTH_WEST,
                PointDelta::SOUTH_EAST,
            ]
        );
        assert_eq!(
            PointDelta::ALL_DIRECTIONS,
            [
                PointDelta::NORTH_WEST,
                PointDelta::NORTH,
                PointDelta::NORTH_EAST,
                PointDelta::WEST,
                PointDelta::EAST,
                PointDelta::SOUTH_WEST,
                PointDelta::SOUTH,
                PointDelta::SOUTH_EAST,
            ]
        );
    }

    #[test]
    fn inverted_reverses_delta() {
        assert_eq!(PointDelta::new(5, -7).inverted(), PointDelta::new(-5, 7));
    }

    #[test]
    fn deltas_add_and_subtract() {
        let lhs = PointDelta::new(4, 1);
        let rhs = PointDelta::new(-2, 5);

        assert_eq!(lhs + rhs, PointDelta::new(2, 6));
        assert_eq!(lhs - rhs, PointDelta::new(6, -4));
    }

    #[test]
    fn delta_add_assign_and_sub_assign_update_in_place() {
        let mut delta = PointDelta::new(1, 1);

        delta += PointDelta::new(2, -3);
        assert_eq!(delta, PointDelta::new(3, -2));

        delta -= PointDelta::new(4, 4);
        assert_eq!(delta, PointDelta::new(-1, -6));
    }

    #[test]
    fn delta_distance_euclid_is_correct() {
        assert_eq!(PointDelta::new(3, 4).distance_euclid(), 5.0);
        assert_eq!(PointDelta::new(-3, -4).distance_euclid(), 5.0);
        assert_eq!(PointDelta::new(0, 0).distance_euclid(), 0.0);
    }

    #[test]
    fn delta_distance_taxicab_is_correct() {
        assert_eq!(PointDelta::new(3, 4).distance_taxicab(), 7);
        assert_eq!(PointDelta::new(-3, -4).distance_taxicab(), 7);
        assert_eq!(PointDelta::new(0, 0).distance_taxicab(), 0);
    }

    #[test]
    fn delta_distance_chebyshev_is_correct() {
        assert_eq!(PointDelta::new(3, 4).distance_chebyshev(), 4);
        assert_eq!(PointDelta::new(-3, -4).distance_chebyshev(), 4);
        assert_eq!(PointDelta::new(0, 0).distance_chebyshev(), 0);
    }
}
