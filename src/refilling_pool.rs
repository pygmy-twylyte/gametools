//! # `RefillingPool` - a randomized pool of items that refills itself when empty.
//! ---
//! A `RefillingPool` is a randomly ordered collection of almost any kind of item.
//! Items drawn from the pool are returned in randomized order until the pool is empty.
//! When the pool is empty, it refills with the original items. It is guaranteed that
//! every item will be drawn once before any are drawn twice, and every twice before any
//! thrice, etc.
//!
//! A `RefillingPool` can never be empty. Attempting to create an empty one or to remove
//! the last item from one will generate a `GameError`.

use rand::seq::SliceRandom as _;

use crate::{GameResult, gameerror::GameError};

/// # `RefillingPool<T>`
///
/// This is an unordered collection of things: letters, numbers, pictures, strings --
/// anything that implements `Clone`. Items drawn from the `RefillingPool` are returned
/// in random order. Once the pool is exhausted, it is automatically refilled with the
/// original set of items. Item order is re-randomized with each refill.
///
/// Note that `RefillingPool` implements `Iterator`, so adapters like `filter`, `map` and
/// `take` can be combined with it some useful ways.
///
/// # Examples
/// Here's a way you might create a set of random but eventually repeating phrases for NPC
/// dialogue:
///
/// ```
/// use gametools::{GameResult, RefillingPool};
///
/// # fn main() -> GameResult<()> {
/// let mut halt_phrases = RefillingPool::new([
///     "Freeze!",
///     "Halt!",
///     "Stop!",
/// ])?;
/// assert_eq!( halt_phrases.full_size(), 3);
/// assert_eq!( halt_phrases.current_size(), 3);
///
/// println!("Guard says: {}", halt_phrases.draw());
///
/// // Guard said any one of the three phrases above, so two are left
/// assert_eq!( halt_phrases.current_size(), 2 );
///
/// let _either_of_the_last_two = halt_phrases.draw();
/// assert_eq!( halt_phrases.current_size(), 1 );
///
/// let _last_remaining = halt_phrases.draw();
/// assert_eq!( halt_phrases.current_size(), 0 );
///
/// let _another_draw_refills_on_demand = halt_phrases.draw();
/// assert_eq!( halt_phrases.current_size(), 2 );
///
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RefillingPool<T> {
    items: Vec<T>,
    unused: Vec<usize>,
}

impl<T> RefillingPool<T> {
    /// Create a new `RefillingPool` from an iterable collection.
    ///
    /// # Errors
    /// - `PoolHasNoRefills` if `items` is empty.
    ///
    /// # Examples
    /// ```
    /// # use gametools::GameResult;
    /// # fn main() -> GameResult<()> {
    /// use gametools::RefillingPool;
    ///
    /// let mut pool = RefillingPool::new(["foo", "bar", "baz"])?;
    /// assert_eq!(pool.full_size(), 3);
    ///
    /// let mut pool = RefillingPool::new(0..5)?;
    /// assert_eq!(pool.full_size(), 5);
    /// # Ok(()) }
    /// ```
    pub fn new(items: impl IntoIterator<Item = T>) -> GameResult<Self> {
        let items: Vec<T> = items.into_iter().collect();
        if items.is_empty() {
            return Err(GameError::PoolCannotBeEmpty);
        }
        let mut unused: Vec<usize> = (0..items.len()).collect();
        unused.shuffle(&mut rand::rng());
        Ok(Self { items, unused })
    }

    /// Add an item to the pool.
    ///
    /// Items added are available to `draw()` immediately and are included in
    /// all subsequent refills.
    ///
    /// # Examples
    /// ```
    /// use gametools::{GameResult, RefillingPool};
    /// # fn main() -> GameResult<()> {
    ///
    /// let mut pool = RefillingPool::new([
    ///     "eenie", "meenie", "miney",
    /// ])?;
    /// assert_eq!( pool.full_size(), 3 );
    /// assert_eq!( pool.current_size(), 3);
    ///
    /// // oh no, we forgot moe!
    /// assert!(!pool.by_ref().take(50).any(|i| i == "moe"));
    /// pool.add("moe");
    /// assert_eq!( pool.full_size(), 4);
    ///
    /// // we could draw at most the new number of items in the full pool before "moe" appears
    /// let size = pool.full_size();
    /// assert!( pool.take(size).any(|i| i == "moe"));
    ///
    /// # Ok(()) }
    pub fn add(&mut self, item: T) {
        self.items.push(item);
        self.unused.push(self.items.len() - 1);
    }

    /// Remove an item from the pool by index.
    ///
    /// # Errors
    /// - `InvalidPoolIndex` if the supplied index is beyond current pool size.
    pub fn remove_index(&mut self, index: usize) -> GameResult<T> {
        if index >= self.items.len() {
            return Err(GameError::InvalidPoolIndex(index, self.items.len()));
        }
        if self.items.len() == 1 {
            return Err(GameError::PoolCannotBeEmpty);
        }
        self.unused.retain(|&ui| ui != index);
        Ok(self.items.swap_remove(index))
    }

    /// Get the size of the pool when new or newly refilled.
    ///
    /// # Examples
    /// ```
    /// # use gametools::{GameResult, RefillingPool};
    /// # fn main() -> GameResult<()> {
    /// let pool = RefillingPool::new(0..3)?;
    /// assert_eq!( pool.full_size(), 3);
    /// # Ok(()) }
    /// ```
    pub fn full_size(&self) -> usize {
        self.items.len()
    }

    /// Get the number of items remaining before a refill is triggered.
    ///
    /// # Examples
    /// ```
    /// # use gametools::{GameResult, RefillingPool};
    /// # fn main() -> GameResult<()> {
    /// let mut pool = RefillingPool::new(0..10)?;
    /// assert_eq!( pool.current_size(), 10 );
    /// let _ = pool.draw();
    /// assert_eq!( pool.current_size(), 9 );
    /// # Ok(()) }
    /// ```
    pub fn current_size(&self) -> usize {
        self.unused.len()
    }

    /// Refill and randomize the unused item list.
    fn refill(&mut self) {
        self.unused = (0..self.items.len()).collect();
        self.unused.shuffle(&mut rand::rng());
    }
}

impl<T: PartialEq> RefillingPool<T> {
    /// Remove an item from the pool, if it exists. If there are multiple copies of the item,
    /// only the first one found is removed.
    ///
    /// # Examples
    /// ```
    /// # use gametools::{GameResult, RefillingPool};
    /// # fn main() -> GameResult<()> {
    /// let mut pool = RefillingPool::new(["Alex", "Geddy", "Neil"])?;
    /// assert_eq!( pool.full_size(), 3 );
    /// assert_eq!( pool.current_size(), 3);
    ///
    /// let rip_professor = pool.remove(&"Neil");
    /// assert_eq!( pool.full_size(), 2);
    /// assert_eq!( pool.current_size(), 2);
    /// assert_eq!( rip_professor, Some("Neil"));
    /// # Ok(()) }
    /// ```
    pub fn remove(&mut self, item: &T) -> Option<T> {
        if let Some(index) = self.items.iter().position(|i| i == item) {
            self.remove_index(index).ok()
        } else {
            None
        }
    }

    /// Determine the index of an item within the `RefillingPool`, if it exists.
    /// If multiple copies of the item are present, the index of the first copy found
    /// is returned.
    ///
    /// # Examples
    /// ```
    /// # use gametools::{GameResult, RefillingPool};
    /// # fn main() -> GameResult<()> {
    /// let pool = RefillingPool::new(["zero", "one", "two"])?;
    /// assert_eq!( pool.index_of(&"one"), Some(1) );
    /// assert!( pool.index_of(&"five").is_none() );
    /// # Ok(()) }
    /// ```
    pub fn index_of(&self, item: &T) -> Option<usize> {
        self.items.iter().position(|i| i == item)
    }
}

impl<T: Clone> RefillingPool<T> {
    /// Draw an item from the pool.
    ///
    /// # Examples
    /// ```
    /// # use gametools::{GameResult, RefillingPool};
    /// # fn main() -> GameResult<()> {
    /// let mut pool = RefillingPool::new(['a', 'b', 'c'])?;
    /// let drawn_letter = pool.draw();
    /// assert!(['a','b','c'].contains(&drawn_letter));
    /// # Ok(()) }
    pub fn draw(&mut self) -> T {
        if self.unused.is_empty() {
            self.refill();
        }
        self.items[self.unused.pop().unwrap()].clone()
    }
}

impl<T: Clone> Iterator for RefillingPool<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.draw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_pool_with_no_refills_is_error() {
        assert!(RefillingPool::<()>::new([]).is_err_and(|e| e.eq(&GameError::PoolCannotBeEmpty)));
    }

    #[test]
    fn removing_last_item_is_error() {
        let mut pool = RefillingPool::new([1]).unwrap();
        assert!(
            pool.remove_index(0)
                .is_err_and(|e| e.eq(&GameError::PoolCannotBeEmpty))
        );
    }

    #[test]
    fn pool_full_and_current_sizes_correct_after_draws_and_refill() {
        let mut pool = RefillingPool::new([1, 2, 3]).unwrap();
        assert_eq!(dbg!(pool.full_size()), 3);
        assert_eq!(dbg!(pool.current_size()), 3);

        pool.draw();
        assert_eq!(pool.full_size(), 3);
        assert_eq!(pool.current_size(), 2);

        pool.draw();
        pool.draw();
        assert_eq!(dbg!(pool.full_size()), 3);
        assert_eq!(dbg!(pool.current_size()), 0);

        // next draw() should trigger refill then immediate draw()
        pool.draw();
        assert_eq!(pool.current_size(), 2);
    }
}
