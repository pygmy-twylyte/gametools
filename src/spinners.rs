//! Spinners Module
//!
//! Implements a game Spinner, comprised of Wedges which can be uniform or of different
//! relative widths, and can be blocked / covered according to game conditions. Wedges
//! may contain numeric values, strings, enums, or other user-defined types.
#[derive(Debug, PartialEq, Clone)]
pub struct Wedge<T> {
    pub value: T,
    pub width: usize,
    pub active: bool,
}
impl<T> Wedge<T> {
    /// Create a new wedge to place on the spinner.
    pub fn new(value: T, width: usize, active: bool) -> Self {
        Self {
            value,
            width,
            active,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spinner<T> {
    pub wedges: Vec<Wedge<T>>,
}

impl<T> Spinner<T> {
    /// Create a new spinner with a vector of wedges.
    pub fn new(wedges: Vec<Wedge<T>>) -> Self {
        Self { wedges }
    }
}

#[cfg(test)]
mod spinner_tests {
    use crate::spinners::*;
    #[test]
    fn can_create_wedges_with_varied_types() {
        let text_wedge = Wedge::new("Winner".to_string(), 1, true);
        assert_eq!(text_wedge.value, "Winner");
        let numeric = Wedge::new(10, 1, true);
        assert_eq!(numeric.value, 10);
    }

    #[test]
    fn can_create_spinners_with_varied_wedge_types() {
        let num_wedges = vec![
            Wedge::new(100, 1, true),
            Wedge::new(200, 1, true),
            Wedge::new(500, 1, true),
        ];
        let numeric_spinner = Spinner::new(num_wedges);
        assert_eq!(numeric_spinner.wedges.len(), 3);
        
        let text_wedges = vec![
            Wedge::new("Lose a Turn".to_string(), 2, true),
            Wedge::new("Ahead 4".to_string(), 4, true),
            Wedge::new("Back 2".to_string(), 4, true),
        ];
        let text_spinner = Spinner::new(text_wedges);
        assert_eq!(text_spinner.wedges.len(), 3);
    }
}
