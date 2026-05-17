//! # `MeteredResource`
//!
//! [`MeteredResource`] models a bounded unsigned counter such as health, stamina,
//! ammunition, or mana. Bounds must satisfy `min < max`; construction validates
//! the initial state, and later increases or reductions clamp at those inclusive
//! bounds instead of wrapping.
//!
//! # Examples
//! ```
//! use gametools::{GameResult, MeteredResource};
//!
//! # fn main() -> GameResult<()> {
//! let mut health = MeteredResource::new("hp", 0_u32, 100, 75)?;
//! assert_eq!(health.current(), 75);
//! assert_eq!(health.fraction_left(), 0.75);
//!
//! health.reduce_by(90);
//! assert_eq!(health.current(), 0);
//! assert!(health.is_depleted());
//!
//! health.increase_by(40);
//! assert_eq!(health.current(), 40);
//!
//! health.refill();
//! assert_eq!(health.current(), 100);
//! assert!(health.is_full());
//! # Ok(()) }
//! ```

use crate::{GameResult, gameerror::ValueError};

mod sealed {
    pub trait Sealed {}

    macro_rules! impl_sealed {
        ($($ty:ty),* $(,)?) => {
            $(impl Sealed for $ty {})*
        };
    }

    impl_sealed!(u8, u16, u32, u64, u128, usize);
}

/// Unsigned primitive integer values that can back a [`MeteredResource`].
///
/// This trait is sealed so `MeteredResource<T>` is limited to Rust's unsigned
/// primitive integer types. Its arithmetic methods are intentionally saturating:
/// a metered resource should clamp at its configured bounds, not wrap around them.
pub trait MeteredValue: sealed::Sealed + Copy + Ord {
    /// Returns `self + rhs`, saturating at the numeric maximum instead of overflowing.
    fn saturating_add(self, rhs: Self) -> Self;

    /// Returns `self - rhs`, saturating at zero instead of underflowing.
    fn saturating_sub(self, rhs: Self) -> Self;

    /// Converts the value to `f64` for ratio calculations.
    fn as_f64(self) -> f64;
}

macro_rules! impl_metered_value {
    ($($ty:ty),* $(,)?) => {
        $(
            impl MeteredValue for $ty {
                fn saturating_add(self, rhs: Self) -> Self {
                    self.saturating_add(rhs)
                }

                fn saturating_sub(self, rhs: Self) -> Self {
                    self.saturating_sub(rhs)
                }

                fn as_f64(self) -> f64 {
                    self as f64
                }
            }
        )*
    };
}

impl_metered_value!(u8, u16, u32, u64, u128, usize);

/// A bounded unsigned resource that can be depleted and refilled.
///
/// `MeteredResource` keeps a current value between a minimum and maximum bound.
/// The bounds must satisfy `min < max`. All supported backing values are unsigned
/// primitive integers such as `u8`, `u32`, and `usize`.
#[derive(Debug, Clone, PartialEq)]
pub struct MeteredResource<T> {
    unit: String,
    min: T,
    max: T,
    current: T,
}

impl<T> MeteredResource<T> {
    /// Returns the unit label attached to the resource.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let health = MeteredResource::new("hp", 0_u32, 100, 75)?;
    /// assert_eq!(health.unit(), "hp");
    /// # Ok(()) }
    /// ```
    pub fn unit(&self) -> &str {
        &self.unit
    }
}

impl<T: Copy> MeteredResource<T> {
    /// Returns the minimum allowed value for the resource.
    pub fn min(&self) -> T {
        self.min
    }

    /// Returns the maximum allowed value for the resource.
    pub fn max(&self) -> T {
        self.max
    }

    /// Returns the current value of the resource.
    pub fn current(&self) -> T {
        self.current
    }

    /// Deplete the resource to its minimum value.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut stamina = MeteredResource::new("stamina", 10_u16, 50, 35)?;
    /// stamina.deplete();
    /// assert_eq!(stamina.current(), 10);
    /// # Ok(()) }
    /// ```
    pub fn deplete(&mut self) {
        self.current = self.min;
    }

    /// Refill the resource to its maximum value.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut stamina = MeteredResource::new("stamina", 10_u16, 50, 35)?;
    /// stamina.refill();
    /// assert_eq!(stamina.current(), 50);
    /// # Ok(()) }
    /// ```
    pub fn refill(&mut self) {
        self.current = self.max;
    }
}

impl<T: MeteredValue> MeteredResource<T> {
    /// Creates a resource with the given unit, minimum, maximum, and current values.
    ///
    /// # Errors
    /// - [`ValueError::MinOverMax`] if `min >= max`.
    /// - [`ValueError::OutOfRange`] if `current` falls outside the inclusive
    ///   range `min..=max`.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mana = MeteredResource::new("mp", 0_u32, 40, 12)?;
    /// assert_eq!(mana.min(), 0);
    /// assert_eq!(mana.max(), 40);
    /// assert_eq!(mana.current(), 12);
    /// # Ok(()) }
    /// ```
    pub fn new(unit: impl ToString, min: T, max: T, current: T) -> GameResult<Self> {
        if min >= max {
            return Err(ValueError::MinOverMax.into());
        }
        if current < min || current > max {
            return Err(ValueError::OutOfRange.into());
        }
        Ok(Self {
            unit: unit.to_string(),
            min,
            max,
            current,
        })
    }

    /// Returns a copy of this resource with new bounds.
    ///
    /// The current value is clamped into the new inclusive range. The original
    /// resource is unchanged.
    ///
    /// # Errors
    /// - [`ValueError::MinOverMax`] if `min >= max`.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let health = MeteredResource::new("hp", 0_u32, 100, 80)?;
    /// let wounded = health.with_new_bounds(0, 50)?;
    ///
    /// assert_eq!(health.current(), 80);
    /// assert_eq!(wounded.max(), 50);
    /// assert_eq!(wounded.current(), 50);
    /// # Ok(()) }
    /// ```
    pub fn with_new_bounds(&self, min: T, max: T) -> GameResult<Self> {
        if min >= max {
            return Err(ValueError::MinOverMax.into());
        }
        let current = match (self.current >= min, self.current <= max) {
            (true, true) => self.current,
            (true, false) => max,
            (false, true) => min,
            (false, false) => unreachable!("can't be both < min and > max"),
        };
        Ok(Self {
            min,
            max,
            current,
            unit: self.unit.clone(),
        })
    }

    /// Returns `true` if the resource is at its minimum value.
    pub fn is_depleted(&self) -> bool {
        self.current <= self.min
    }

    /// Returns `true` if the resource is at its maximum value.
    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    /// Returns the fraction of the resource currently available.
    ///
    /// The result is always between `0.0` and `1.0`, inclusive.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let stamina = MeteredResource::new("stamina", 20_u32, 120, 70)?;
    /// assert_eq!(stamina.fraction_left(), 0.5);
    /// # Ok(()) }
    /// ```
    pub fn fraction_left(&self) -> f64 {
        self.current.saturating_sub(self.min).as_f64() / self.max.saturating_sub(self.min).as_f64()
    }

    /// Reduces the resource by the given amount, clamping to the minimum value if necessary.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut arrows = MeteredResource::new("arrows", 0_u8, 20, 5)?;
    /// arrows.reduce_by(9);
    /// assert_eq!(arrows.current(), 0);
    /// # Ok(()) }
    /// ```
    pub fn reduce_by(&mut self, amount: T) {
        self.current = self.current.saturating_sub(amount).max(self.min);
    }

    /// Increases the resource by the given amount, clamping to the maximum value if necessary.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, MeteredResource};
    ///
    /// # fn main() -> GameResult<()> {
    /// let mut arrows = MeteredResource::new("arrows", 0_u8, 20, 18)?;
    /// arrows.increase_by(9);
    /// assert_eq!(arrows.current(), 20);
    /// # Ok(()) }
    /// ```
    pub fn increase_by(&mut self, amount: T) {
        self.current = self.current.saturating_add(amount).min(self.max);
    }
}

#[cfg(test)]
mod tests {
    use super::MeteredResource;
    use crate::{GameError, gameerror::ValueError};

    #[test]
    fn new_rejects_non_increasing_bounds() {
        assert_eq!(
            MeteredResource::new("hp", 10_u8, 10, 10),
            Err(GameError::ValueError(ValueError::MinOverMax))
        );
        assert_eq!(
            MeteredResource::new("hp", 11_u8, 10, 10),
            Err(GameError::ValueError(ValueError::MinOverMax))
        );
    }

    #[test]
    fn new_rejects_current_values_outside_bounds() {
        assert_eq!(
            MeteredResource::new("hp", 10_u8, 20, 9),
            Err(GameError::ValueError(ValueError::OutOfRange))
        );
        assert_eq!(
            MeteredResource::new("hp", 10_u8, 20, 21),
            Err(GameError::ValueError(ValueError::OutOfRange))
        );
    }

    #[test]
    fn with_new_bounds_clamps_without_changing_original() {
        let resource = MeteredResource::new("hp", 0_u8, 100, 80).unwrap();

        let lower_max = resource.with_new_bounds(0, 50).unwrap();
        let higher_min = resource.with_new_bounds(90, 120).unwrap();

        assert_eq!(resource.current(), 80);
        assert_eq!(lower_max.current(), 50);
        assert_eq!(higher_min.current(), 90);
    }

    #[test]
    fn with_new_bounds_rejects_non_increasing_bounds() {
        let resource = MeteredResource::new("hp", 0_u8, 100, 80).unwrap();

        assert_eq!(
            resource.with_new_bounds(50, 50),
            Err(GameError::ValueError(ValueError::MinOverMax))
        );
    }

    #[test]
    fn deplete_refill_and_state_queries_use_configured_bounds() {
        let mut resource = MeteredResource::new("stamina", 10_u8, 20, 15).unwrap();

        resource.deplete();
        assert!(resource.is_depleted());
        assert!(!resource.is_full());

        resource.refill();
        assert!(resource.is_full());
        assert!(!resource.is_depleted());
    }

    #[test]
    fn reduce_by_clamps_at_minimum_without_underflowing() {
        let mut resource = MeteredResource::new("hp", 10_u8, 100, 25).unwrap();

        resource.reduce_by(30);

        assert_eq!(resource.current(), 10);
    }

    #[test]
    fn increase_by_clamps_at_maximum_without_overflowing() {
        let mut resource = MeteredResource::new("mp", 0_u8, 250, 240).unwrap();

        resource.increase_by(30);

        assert_eq!(resource.current(), 250);
    }

    #[test]
    fn fraction_full_uses_the_configured_range() {
        let resource = MeteredResource::new("stamina", 20_u32, 120, 70).unwrap();

        assert_eq!(resource.fraction_left(), 0.5);
    }
}
