//! Priority queue utilities for retrieving only the next highest- or lowest-priority item.
//!
//! `PriorityQueue` is backed by a `BinaryHeap`, so it is the better fit when you
//! care about fast insertion and removal of the next item rather than inspecting
//! the full ordering of the collection.
//!
//! # Examples
//!
//! ```
//! use gametools::ordering::{MaxPriorityQ, MinPriorityQ};
//!
//! let mut max_q = MaxPriorityQ::new();
//! max_q.push("low", 1);
//! max_q.push("high", 3);
//!
//! assert_eq!(max_q.pop(), Some(("high", 3)));
//!
//! let mut min_q = MinPriorityQ::new();
//! min_q.push("late", 10);
//! min_q.push("soon", 5);
//!
//! assert_eq!(min_q.pop(), Some(("soon", 5)));
//! ```

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::marker::PhantomData;

/// A [`PriorityQueue`] that yields the highest-priority items first.
pub type MaxPriorityQ<P, T> = PriorityQueue<P, T, Max>;
/// A [`PriorityQueue`] that yields the lowest-priority items first.
pub type MinPriorityQ<P, T> = PriorityQueue<P, T, Min>;

/// Marker type for [`PriorityQueue`] values that yield the highest-priority items first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Max;
/// Marker type for [`PriorityQueue`] values that yield the lowest-priority items first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Min;

/// A priority queue with stable tie-breaking by insertion order.
///
/// This queue returns only the next item in priority order. When two items share
/// the same priority, the item that was inserted first is returned first.
///
/// # Type Parameters
/// - `P: Ord` - the priority value type.
/// - `T` - the stored item type.
/// - `O = Max` - the ordering marker, typically [`Max`] or [`Min`].
///
/// # Aliases
/// - [`MaxPriorityQ<P, T>`]: highest priorities first.
/// - [`MinPriorityQ<P, T>`]: lowest priorities first.
///
/// # Examples
///
/// ```
/// use gametools::ordering::{PriorityQueue, Min};
///
/// let mut queue = PriorityQueue::<_, _, Min>::new();
/// queue.push("boss", 10);
/// queue.push("minion", 1);
///
/// assert_eq!(queue.pop(), Some(("minion", 1)));
/// assert_eq!(queue.pop(), Some(("boss", 10)));
/// ```
#[derive(Debug, Default, Clone)]
pub struct PriorityQueue<P, T, O = Max>
where
    P: Ord,
{
    heap: BinaryHeap<RankedItem<P, T, O>>,
    seq: u64,
}
impl<P: Ord, T, O> PriorityQueue<P, T, O> {
    /// Creates an empty `PriorityQueue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::{MaxPriorityQ, PriorityQueue, Min};
    ///
    /// let mut max_q = MaxPriorityQ::new();
    /// max_q.push("token", 1);
    ///
    /// let mut min_q = PriorityQueue::<_, _, Min>::new();
    /// min_q.push("token", 1);
    ///
    /// assert_eq!(max_q.pop(), Some(("token", 1)));
    /// assert_eq!(min_q.pop(), Some(("token", 1)));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            seq: 0,
        }
    }

    fn pop_inner(&mut self) -> Option<(T, P)>
    where
        O: QueueOrder,
    {
        self.heap
            .pop()
            .map(|RankedItem { item, priority, .. }| (item, priority))
    }

    fn push_inner(&mut self, item: T, priority: P)
    where
        O: QueueOrder,
    {
        self.seq += 1;
        self.heap.push(RankedItem {
            item,
            priority,
            seq: self.seq,
            _order: PhantomData,
        });
    }
}

impl<P: Ord, T> PriorityQueue<P, T, Max> {
    /// Removes and returns the highest-priority item.
    ///
    /// Returns `None` if the queue is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::MaxPriorityQ;
    ///
    /// let mut queue = MaxPriorityQ::new();
    /// queue.push("low", 1);
    /// queue.push("high", 2);
    ///
    /// assert_eq!(queue.pop(), Some(("high", 2)));
    /// assert_eq!(queue.pop(), Some(("low", 1)));
    /// assert_eq!(queue.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<(T, P)> {
        self.pop_inner()
    }

    /// Inserts an item with the given priority.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::MaxPriorityQ;
    ///
    /// let mut queue = MaxPriorityQ::new();
    /// queue.push("alpha", 1);
    /// queue.push("beta", 2);
    ///
    /// assert_eq!(queue.pop(), Some(("beta", 2)));
    /// ```
    pub fn push(&mut self, item: T, priority: P) {
        self.push_inner(item, priority);
    }
}

impl<P: Ord, T> PriorityQueue<P, T, Min> {
    /// Removes and returns the lowest-priority item.
    ///
    /// Returns `None` if the queue is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::MinPriorityQ;
    ///
    /// let mut queue = MinPriorityQ::new();
    /// queue.push("late", 3);
    /// queue.push("soon", 1);
    ///
    /// assert_eq!(queue.pop(), Some(("soon", 1)));
    /// assert_eq!(queue.pop(), Some(("late", 3)));
    /// assert_eq!(queue.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<(T, P)> {
        self.pop_inner()
    }

    /// Inserts an item with the given priority.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::MinPriorityQ;
    ///
    /// let mut queue = MinPriorityQ::new();
    /// queue.push("late", 3);
    /// queue.push("soon", 1);
    ///
    /// assert_eq!(queue.pop(), Some(("soon", 1)));
    /// ```
    pub fn push(&mut self, item: T, priority: P) {
        self.push_inner(item, priority);
    }
}

/// Ordering strategy used internally by [`PriorityQueue`] to support max-heap
/// and min-heap behavior with the same underlying `BinaryHeap`.
trait QueueOrder {
    /// Compares two `(priority, insertion-sequence)` pairs.
    fn cmp<P: Ord>(lhs_priority: &P, lhs_seq: u64, rhs_priority: &P, rhs_seq: u64) -> Ordering;
}

impl QueueOrder for Max {
    fn cmp<P: Ord>(lhs_priority: &P, lhs_seq: u64, rhs_priority: &P, rhs_seq: u64) -> Ordering {
        lhs_priority.cmp(rhs_priority).then(rhs_seq.cmp(&lhs_seq))
    }
}

impl QueueOrder for Min {
    fn cmp<P: Ord>(lhs_priority: &P, lhs_seq: u64, rhs_priority: &P, rhs_seq: u64) -> Ordering {
        lhs_priority
            .cmp(rhs_priority)
            .reverse()
            .then(rhs_seq.cmp(&lhs_seq))
    }
}

/// An item stored in the heap with its priority and insertion sequence number.
#[derive(Debug, Clone)]
struct RankedItem<P, T, O>
where
    P: Ord,
{
    item: T,
    priority: P,
    seq: u64,
    _order: PhantomData<O>,
}

impl<P, T, O> Ord for RankedItem<P, T, O>
where
    P: Ord,
    O: QueueOrder,
{
    fn cmp(&self, other: &Self) -> Ordering {
        O::cmp(&self.priority, self.seq, &other.priority, other.seq)
    }
}

impl<P, T, O> PartialOrd for RankedItem<P, T, O>
where
    P: Ord,
    O: QueueOrder,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P, T, O> Eq for RankedItem<P, T, O>
where
    P: Ord,
    O: QueueOrder,
{
}

impl<P, T, O> PartialEq for RankedItem<P, T, O>
where
    P: Ord,
    O: QueueOrder,
{
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.seq == other.seq
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push_pop_max() {
        let mut q = MaxPriorityQ::new();
        q.push('c', 3);
        q.push('a', 1);
        q.push('b', 2);
        assert_eq!(q.pop(), Some(('c', 3)));
        assert_eq!(q.pop(), Some(('b', 2)));
        assert_eq!(q.pop(), Some(('a', 1)));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn test_push_pop_min() {
        let mut q = MinPriorityQ::new();
        q.push('c', 3);
        q.push('a', 1);
        q.push('b', 2);
        assert_eq!(q.pop(), Some(('a', 1)));
        assert_eq!(q.pop(), Some(('b', 2)));
        assert_eq!(q.pop(), Some(('c', 3)));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn ties_returned_in_insertion_order() {
        let mut q = MaxPriorityQ::new();
        q.push('a', 3);
        q.push('b', 3);
        q.push('c', 3);
        assert_eq!(q.pop(), Some(('a', 3)));
        assert_eq!(q.pop(), Some(('b', 3)));
        assert_eq!(q.pop(), Some(('c', 3)));
        assert_eq!(q.pop(), None);

        let mut q = MinPriorityQ::new();
        q.push('a', 3);
        q.push('b', 3);
        q.push('c', 3);
        assert_eq!(q.pop(), Some(('a', 3)));
        assert_eq!(q.pop(), Some(('b', 3)));
        assert_eq!(q.pop(), Some(('c', 3)));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn empty_queues_pop_none() {
        let mut max_q: MaxPriorityQ<i32, char> = MaxPriorityQ::new();
        let mut min_q: MinPriorityQ<i32, char> = MinPriorityQ::new();

        assert_eq!(max_q.pop(), None);
        assert_eq!(min_q.pop(), None);
    }

    #[test]
    fn explicit_generic_construction_respects_order_marker() {
        let mut max_q = PriorityQueue::<_, _, Max>::new();
        max_q.push("mid", 2);
        max_q.push("high", 3);
        max_q.push("low", 1);

        assert_eq!(max_q.pop(), Some(("high", 3)));
        assert_eq!(max_q.pop(), Some(("mid", 2)));
        assert_eq!(max_q.pop(), Some(("low", 1)));

        let mut min_q = PriorityQueue::<_, _, Min>::new();
        min_q.push("mid", 2);
        min_q.push("high", 3);
        min_q.push("low", 1);

        assert_eq!(min_q.pop(), Some(("low", 1)));
        assert_eq!(min_q.pop(), Some(("mid", 2)));
        assert_eq!(min_q.pop(), Some(("high", 3)));
    }
}
