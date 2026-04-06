//! `RefillingPool` module
//!
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

pub struct RefillingPool<T> {
    items: Vec<T>,
    unused: Vec<usize>,
}

impl<T> RefillingPool<T> {
    /// Create a new `RefillingPool` from a collection.
    ///
    /// # Errors
    /// - `PoolHasNoRefills` if `items` is empty.
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

    /// Get the size of the pool when newly refilled.
    pub fn full_size(&self) -> usize {
        self.items.len()
    }

    /// Get the number of items remaining before the pool refills.
    pub fn current_size(&self) -> usize {
        self.unused.len()
    }

    /// Refill and randomize the unused item list.
    fn refill(&mut self) {
        self.unused = (0..self.items.len()).collect();
        self.unused.shuffle(&mut rand::rng());
    }
}

impl<T: Clone> RefillingPool<T> {
    /// Draw an item from the pool.
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
    fn pool_full_and_current_size_correct_after_draws_and_refill() {
        let mut pool = RefillingPool::new([1, 2, 3]).unwrap();
        assert_eq!(dbg!(pool.full_size()), 3);
        assert_eq!(dbg!(pool.current_size()), 3);
        let _ = pool.draw();
        assert_eq!(dbg!(pool.full_size()), 3);
        assert_eq!(dbg!(pool.current_size()), 2);
        pool.draw();
        pool.draw();
        assert_eq!(dbg!(pool.full_size()), 3);
        assert_eq!(dbg!(pool.current_size()), 0);
    }
}
