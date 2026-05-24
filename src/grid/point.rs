//! # `Point`

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub col: i32,
    pub row: i32,
}

macro_rules! delta {
    ($dc:expr, $dr:expr) => {
        PointDelta { dc: $dc, dr: $dr }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PointDelta {
    pub dc: i32,
    pub dr: i32,
}
impl PointDelta {
    pub const NORTH: Self = delta!(0, -1);
    pub const SOUTH: Self = delta!(0, 1);
    pub const EAST: Self = delta!(1, 0);
    pub const WEST: Self = delta!(-1, 0);
    pub const NORTH_EAST: Self = delta!(1, -1);
    pub const NORTH_WEST: Self = delta!(-1, -1);
    pub const SOUTH_EAST: Self = delta!(1, 1);
    pub const SOUTH_WEST: Self = delta!(-1, 1);
}
