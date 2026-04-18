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
    /// - `GameError::PoolCannotBeEmpty` if `items` is empty.
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
        self.unused.shuffle(&mut rand::rng());
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
        let item = self.items.swap_remove(index);
        self.refill();
        Ok(item)
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
    #[must_use]
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
    #[must_use]
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
    ///
    #[allow(
        clippy::missing_panics_doc,
        reason = "cannot panic, refills before pop()"
    )]
    pub fn draw(&mut self) -> T {
        if self.unused.is_empty() {
            self.refill();
        }
        self.items[self.unused.pop().unwrap()].clone()
    }

    /// Draw an item from the pool, but filtered by a predicate.
    ///
    /// When drawing from the pool in this form, it **may not be an infinite source**:
    /// if no `unused` items in the current refill cycle pass the predicate, `None` is
    /// returned.
    ///
    /// # Examples
    /// ```
    /// # use gametools::{GameResult, RefillingPool};
    /// # fn main() -> GameResult<()> {
    /// let mut pool = RefillingPool::new(['a', 'b', 'c'])?;
    ///
    /// let drawn_letter = pool.draw_where(|c| *c == 'a');
    /// assert_eq!(drawn_letter, Some('a'));
    ///
    /// let no_a_left_until_refill = pool.draw_where(|c| *c == 'a');
    /// assert_eq!(no_a_left_until_refill, None);
    /// # Ok(()) }
    ///
    pub fn draw_where<F>(&mut self, mut pred: F) -> Option<T>
    where
        F: FnMut(&T) -> bool,
    {
        if self.unused.is_empty() {
            self.refill();
        }

        let passing_unused_idx = self.unused.iter().position(|i| pred(&self.items[*i]))?;
        let passing_item_idx = self.unused.swap_remove(passing_unused_idx);
        Some(self.items[passing_item_idx].clone())
    }

    /// Draw an item from the pool, but filtered by a predicate using supplied context.
    ///
    /// When drawing using this method, **it is not guaranteed to be an infinite source**:
    /// If none of the unused items in the current refill cycle match the predicate in the
    /// given `context`, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gametools::RefillingPool;
    /// # use gametools::GameResult;
    /// # fn main() -> GameResult<()> {
    ///
    /// let mut pool = RefillingPool::new([1, 2, 3, 4])?;
    ///
    /// let mode = 2;
    /// let chooser = |c: &i32, i: &i32| *i % *c == 0;
    ///
    /// let even = pool.draw_with_context(&mode, chooser);
    /// assert!(even.is_some());
    ///
    /// let even = pool.draw_with_context(&mode, chooser);
    /// assert!(even.is_some());
    ///
    /// let even = pool.draw_with_context(&mode, chooser);
    /// assert!(even.is_none());
    /// assert_eq!(pool.current_size(), 2);
    /// # Ok(()) }
    /// ```
    pub fn draw_with_context<C, F>(&mut self, context: &C, mut chooser: F) -> Option<T>
    where
        F: FnMut(&C, &T) -> bool,
    {
        if self.unused.is_empty() {
            self.refill();
        }

        let passing_unused_idx = self
            .unused
            .iter()
            .position(|i| chooser(context, &self.items[*i]))?;
        let passing_item_idx = self.unused.swap_remove(passing_unused_idx);
        Some(self.items[passing_item_idx].clone())
    }

    /// Draw a preferred item from the `RefillingPool`, falling back to a random item from the
    /// pool if there are none available.
    ///
    /// Preferred items can be defined by specifying a context and a rule (function) to evaluate
    /// the yet-unused items for "preferred" status.
    ///
    /// # Examples
    ///
    /// ```
    /// # use gametools::{RefillingPool, GameResult};
    /// enum WhichFirst {
    ///     Even,
    ///     Odd,
    /// }
    /// # fn main() -> GameResult<()> {
    ///
    /// let mut pool = RefillingPool::new([1,2,3])?;
    ///
    /// // our context (mode) and a function to evaluate items with it
    /// let mode = WhichFirst::Even;
    /// let chooser = |context: &WhichFirst, item: &u32| match context {
    ///     WhichFirst::Even => item.is_multiple_of(2),
    ///     WhichFirst::Odd => !item.is_multiple_of(2),
    /// };
    ///
    /// let preferred = pool.draw_with_context_or_any(&mode, chooser);
    /// // 2 is the only even #, so must be returned first
    /// assert_eq!(preferred, 2);
    ///
    /// let no_even_left = pool.draw_with_context_or_any(&mode, chooser);
    /// // no preferred item was left, so anything else left in the pool is returned
    /// assert!([1,3].contains(&no_even_left));
    /// # Ok(())
    /// # }
    /// ```
    pub fn draw_with_context_or_any<C, F>(&mut self, context: &C, chooser: F) -> T
    where
        F: FnMut(&C, &T) -> bool,
    {
        self.draw_with_context(context, chooser)
            .unwrap_or_else(|| self.draw())
    }
}

impl<T: Clone> Iterator for RefillingPool<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.draw())
    }
}

impl<T: Clone> TryFrom<&[T]> for RefillingPool<T> {
    type Error = GameError;

    fn try_from(items: &[T]) -> Result<Self, Self::Error> {
        RefillingPool::new(items.to_vec())
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

    #[test]
    fn test_refilling_pool_add() {
        let mut pool = RefillingPool::new([1, 2, 3]).unwrap();
        for _ in 1..=300 {
            assert!([1, 2, 3].contains(&pool.draw()))
        }
        pool.add(4);
        assert_eq!(pool.full_size(), 4);
        for _ in 1..=300 {
            assert!([1, 2, 3, 4].contains(&pool.draw()))
        }
    }

    #[test]
    fn test_refilling_pool_remove_index() {
        let mut pool = RefillingPool::new([1, 2, 3]).unwrap();
        let _ = pool.remove_index(0);
        assert_eq!(pool.full_size(), 2);
        assert_eq!(pool.current_size(), 2);
        assert!(pool.take(500).all(|item| item != 1));
    }

    #[test]
    fn test_refilling_pool_index_of() {
        let pool = RefillingPool::new([1, 2, 3]).unwrap();
        assert_eq!(pool.index_of(&1), Some(0));
        assert_eq!(pool.index_of(&2), Some(1));
        assert_eq!(pool.index_of(&3), Some(2));
        assert_eq!(pool.index_of(&4), None);
    }

    #[test]
    fn test_refilling_pool_remove() {
        let mut pool = RefillingPool::new([1, 2, 3]).unwrap();
        let _ = pool.remove(&1);
        assert_eq!(pool.full_size(), 2);
        assert_eq!(pool.current_size(), 2);
        assert!(pool.take(500).all(|item| item != 1));
    }
}
