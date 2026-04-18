//! `PriorityQueue` - a collection of items that can be handled in a prioritized order.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::marker::PhantomData;

pub type MaxPriorityQ<P, T> = PriorityQueue<P, T, Max>;
pub type MinPriorityQ<P, T> = PriorityQueue<P, T, Min>;

/// Type used to indicate that the `PriorityQueue` should yield the highest priority items first.
pub struct Max;
/// Type used to indicate that the `PriorityQueue` should yield the lowest priority items first.
pub struct Min;

/// # `PriorityQueue<P, T, O = Max>`
///
/// A collection that yields its items in a ranked order. If two or more items tie on
/// priority, they are returned in the order they were inserted.
///
/// # Type Parameters
/// - `P: Ord` - the type (typically numeric) of the priority values
/// - `T` - the type of the items in the queue
/// - `O = Max` - determines whether the `Max` (default) or `Min` items are yielded first.
///
/// # Aliases
/// - `MaxPriorityQ<P, T>`: A priority queue that yields the highest priority items first.
/// - `MinPriorityQ<P, T>`: A priority queue that yields the lowest priority items first.
///
/// # Examples
///
/// ```
/// use gametools::ordering::priority_queue::{MaxPriorityQ, MinPriorityQ};
///
/// let mut max_q = MaxPriorityQ::new();
/// let mut min_q = MinPriorityQ::new();
///
/// max_q.push("high", 10);
/// max_q.push("low", 5);
///
/// min_q.push("high", 10);
/// min_q.push("low", 5);
///
/// assert_eq!(max_q.pop(), Some(("high", 10)));
/// assert_eq!(min_q.pop(), Some(("low", 5)));
/// ```
#[derive(Debug)]
pub struct PriorityQueue<P, T, O = Max>
where
    P: Ord,
{
    heap: BinaryHeap<RankedItem<P, T, O>>,
    seq: u64,
}
impl<P: Ord, T, O> PriorityQueue<P, T, O> {
    /// Create a new, empty `PriorityQueue`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::priority_queue::{MaxPriorityQ, PriorityQueue, Min};
    ///
    /// let mut q_high_first = MaxPriorityQ::new();
    /// q_high_first.push("foo", 1);
    ///
    /// // or more verbose / explicit
    /// let mut q_low_first = PriorityQueue::<_, _, Min>::new();
    /// q_low_first.push(2,2);
    /// ```
    ///
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
    /// Take the highest ranked item (and assigned priority value) from the queue.
    pub fn pop(&mut self) -> Option<(T, P)> {
        self.pop_inner()
    }

    /// Add an item to the queue with the given priority.
    pub fn push(&mut self, item: T, priority: P) {
        self.push_inner(item, priority);
    }
}

impl<P: Ord, T> PriorityQueue<P, T, Min> {
    /// Take the lowest ranked item (and assigned priority value) from the queue.
    pub fn pop(&mut self) -> Option<(T, P)> {
        self.pop_inner()
    }

    /// Add an item to the queue with the given priority.
    pub fn push(&mut self, item: T, priority: P) {
        self.push_inner(item, priority);
    }
}

/// Trait used to define ordering for `RankedItem<P, T, O: QueueOrder>` and enable alternative
/// min-heap behavior from `std::collections::BinaryHeap`.
trait QueueOrder {
    /// Determine ordering between two items based on priority and sequence (insertion) order.
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

/// An item on the `PriorityQueue` with a defined `priority` and sequence number.
#[derive(Debug)]
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
}
