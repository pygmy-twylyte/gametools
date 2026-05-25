//! # PointDelta
//!
//! `PointDelta` is a struct representing a delta (change) in position on a 2D grid.

use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct PointDelta {
    pub dc: i32,
    pub dr: i32,
}

macro_rules! delta {
    ($dc:expr, $dr:expr) => {
        PointDelta { dc: $dc, dr: $dr }
    };
}

impl PointDelta {
    pub const fn new(dc: i32, dr: i32) -> Self {
        Self { dc, dr }
    }

    pub const fn inverted(self) -> Self {
        Self::new(-self.dc, -self.dr)
    }

    pub const NORTH: Self = delta!(0, -1);
    pub const SOUTH: Self = delta!(0, 1);
    pub const EAST: Self = delta!(1, 0);
    pub const WEST: Self = delta!(-1, 0);
    pub const NORTH_EAST: Self = delta!(1, -1);
    pub const NORTH_WEST: Self = delta!(-1, -1);
    pub const SOUTH_EAST: Self = delta!(1, 1);
    pub const SOUTH_WEST: Self = delta!(-1, 1);

    pub const DIAGONALS: [Self; 4] = [
        Self::NORTH_EAST,
        Self::NORTH_WEST,
        Self::SOUTH_EAST,
        Self::SOUTH_WEST,
    ];

    pub const CARDINALS: [Self; 4] = [Self::NORTH, Self::SOUTH, Self::EAST, Self::WEST];

    pub const ALL_DIRECTIONS: [Self; 8] = [
        Self::NORTH,
        Self::SOUTH,
        Self::EAST,
        Self::WEST,
        Self::NORTH_EAST,
        Self::NORTH_WEST,
        Self::SOUTH_EAST,
        Self::SOUTH_WEST,
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
                PointDelta::SOUTH,
                PointDelta::EAST,
                PointDelta::WEST,
            ]
        );
        assert_eq!(
            PointDelta::DIAGONALS,
            [
                PointDelta::NORTH_EAST,
                PointDelta::NORTH_WEST,
                PointDelta::SOUTH_EAST,
                PointDelta::SOUTH_WEST,
            ]
        );
        assert_eq!(
            PointDelta::ALL_DIRECTIONS,
            [
                PointDelta::NORTH,
                PointDelta::SOUTH,
                PointDelta::EAST,
                PointDelta::WEST,
                PointDelta::NORTH_EAST,
                PointDelta::NORTH_WEST,
                PointDelta::SOUTH_EAST,
                PointDelta::SOUTH_WEST,
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
}
