//! Stable ranked ordering utilities.
//!
//! `RankedOrder` stores the full ordered list of items instead of optimizing only
//! for the next item to be removed. That makes it useful when you need to inspect
//! or materialize the whole ordering while still preserving insertion order for
//! ties.
//!
//! # Examples
//!
//! ```
//! use gametools::ordering::{AscendingOrder, DescendingOrder};
//!
//! let mut descending = DescendingOrder::new();
//! descending.push("low", 1);
//! descending.push("high", 3);
//! descending.push("also high", 3);
//!
//! assert_eq!(descending.pop(), Some(("high", 3)));
//! assert_eq!(descending.pop(), Some(("also high", 3)));
//!
//! let mut ascending = AscendingOrder::new();
//! ascending.push("medium", 2);
//! ascending.push("low", 1);
//! ascending.push("high", 3);
//!
//! assert_eq!(ascending.into_sorted_vec(), vec![("low", 1), ("medium", 2), ("high", 3)]);
//! ```
/// A [`RankedOrder`] that yields the lowest-ranked items first.
pub type AscendingOrder<R, T> = RankedOrder<R, T, Ascending>;
/// A [`RankedOrder`] that yields the highest-ranked items first.
pub type DescendingOrder<R, T> = RankedOrder<R, T, Descending>;

/// A stable ranked collection that can be viewed or consumed in sorted order.
///
/// The `D` type parameter controls whether lower ranks are yielded first
/// ([`Ascending`]) or higher ranks are yielded first ([`Descending`]).
/// When two items have the same rank, they are yielded in insertion order.
///
/// # Type Parameters
/// - `R: Ord` - the rank value used to sort items.
/// - `T` - the stored item type.
/// - `D: Direction` - the ordering direction used for ranked comparisons.
///
/// # Aliases
/// - [`AscendingOrder<R, T>`]: lower ranks first.
/// - [`DescendingOrder<R, T>`]: higher ranks first.
///
/// # Examples
///
/// ```
/// use gametools::ordering::{Descending, RankedOrder};
///
/// let mut order = RankedOrder::<_, _, Descending>::new();
/// order.push("silver", 2);
/// order.push("gold", 3);
///
/// assert_eq!(order.pop(), Some(("gold", 3)));
/// assert_eq!(order.pop(), Some(("silver", 2)));
/// ```
#[derive(Debug, Default)]
pub struct RankedOrder<R, T, D>
where
    D: Direction,
    R: Ord,
{
    items: Vec<RankedItem<R, T, D>>,
    seq: u64,
    is_dirty: bool,
}

impl<R, T, D> RankedOrder<R, T, D>
where
    D: Direction,
    R: Ord,
{
    /// Creates an empty `RankedOrder`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push("first", 1);
    ///
    /// assert_eq!(order.pop(), Some(("first", 1)));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            seq: 0,
            is_dirty: false,
        }
    }

    /// Inserts an item with an explicit rank value.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::DescendingOrder;
    ///
    /// let mut order = DescendingOrder::new();
    /// order.push("bronze", 1);
    /// order.push("gold", 3);
    ///
    /// assert_eq!(order.pop(), Some(("gold", 3)));
    /// ```
    pub fn push(&mut self, item: T, rank: R) {
        self.push_inner(item, rank);
    }

    /// Inserts an item after computing its rank from the item itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::DescendingOrder;
    ///
    /// let mut order = DescendingOrder::new();
    /// order.push_with_ranker("wizard", |name| name.len());
    /// order.push_with_ranker("rogue", |name| name.len());
    ///
    /// assert_eq!(order.pop(), Some(("wizard", 6)));
    /// assert_eq!(order.pop(), Some(("rogue", 5)));
    /// ```
    pub fn push_with_ranker<F: FnOnce(&T) -> R>(&mut self, item: T, ranker: F) {
        let rank = ranker(&item);
        self.push_inner(item, rank);
    }

    /// Removes and returns the next item in ranked order.
    ///
    /// Returns `None` if the collection is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push("later", 2);
    /// order.push("sooner", 1);
    ///
    /// assert_eq!(order.pop(), Some(("sooner", 1)));
    /// assert_eq!(order.pop(), Some(("later", 2)));
    /// assert_eq!(order.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<(T, R)> {
        self.pop_inner()
    }

    /// Peek at the next item in ranked order without consuming it.
    ///
    /// Returns `None` if the collection is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push("later", 2);
    /// order.push("sooner", 1);
    ///
    /// assert_eq!(order.peek(), Some((&"sooner", &1)));
    /// assert_eq!(order.peek(), Some((&"sooner", &1)));
    /// ```
    #[must_use]
    pub fn peek(&mut self) -> Option<(&T, &R)> {
        self.lazy_sort();
        self.items.last().map(|ri| (&ri.item, &ri.rank))
    }

    /// Returns the number of items in the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push("later", 2);
    /// order.push("sooner", 1);
    ///
    /// assert_eq!(order.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the collection is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push("later", 2);
    /// order.push("sooner", 1);
    ///
    /// assert!(!order.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the minimum and maximum rank in the collection, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push('c', 3);
    /// order.push('a', 1);
    /// order.push('b', 2);
    ///
    /// assert_eq!(order.rank_range(), Some((1, 3)));
    /// ```
    #[must_use]
    pub fn rank_range(&self) -> Option<(R, R)>
    where
        R: Copy,
    {
        let init = self.items.first().map(|ri| ri.rank)?;
        let range = self
            .items
            .iter()
            .skip(1)
            .map(|ri| ri.rank)
            .fold((init, init), |acc, rank| (acc.0.min(rank), acc.1.max(rank)));
        Some(range)
    }

    /// Re-ranks all items using the given ranker function.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push('c', 3);
    /// order.push('a', 1);
    /// order.push('b', 2);
    ///
    /// order.rerank_all_by(|c| *c as u8);
    ///
    /// assert_eq!(order.into_sorted_vec(), vec![('a', 97), ('b', 98), ('c', 99)]);
    /// ```
    pub fn rerank_all_by<F: Fn(&T) -> R>(&mut self, ranker: F) {
        for item in self.items.iter_mut() {
            item.rank = ranker(&item.item);
        }
        self.is_dirty = true;
    }

    /// Consumes the collection and returns all items as a sorted vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::AscendingOrder;
    ///
    /// let mut order = AscendingOrder::new();
    /// order.push('c', 3);
    /// order.push('a', 1);
    /// order.push('b', 2);
    ///
    /// assert_eq!(order.into_sorted_vec(), vec![('a', 1), ('b', 2), ('c', 3)]);
    /// ```
    #[must_use]
    pub fn into_sorted_vec(mut self) -> Vec<(T, R)> {
        self.lazy_sort();
        self.items
            .into_iter()
            .rev()
            .map(|ri| (ri.item, ri.rank))
            .collect()
    }

    /// Returns an iterator over the items in ranked order without consuming them.
    ///
    /// The iterator yields references to `(item, rank)` pairs. This method takes
    /// `&mut self` because sorting is deferred until the ordered view is needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use gametools::ordering::DescendingOrder;
    ///
    /// let mut order = DescendingOrder::new();
    /// order.push("bronze", 1);
    /// order.push("silver", 2);
    /// order.push("gold", 3);
    ///
    /// assert_eq!(
    ///     order.iter_sorted().collect::<Vec<_>>(),
    ///     vec![(&"gold", &3), (&"silver", &2), (&"bronze", &1)]
    /// );
    /// ```
    pub fn iter_sorted(&mut self) -> impl Iterator<Item = (&T, &R)> {
        self.lazy_sort();
        self.items.iter().rev().map(|ri| (&ri.item, &ri.rank))
    }

    fn push_inner(&mut self, item: T, rank: R) {
        self.seq = self.seq.wrapping_add(1);
        self.items.push(RankedItem::new(item, rank, self.seq));
        self.is_dirty = true;
    }

    fn pop_inner(&mut self) -> Option<(T, R)> {
        self.lazy_sort();
        self.items.pop().map(|ri| (ri.item, ri.rank))
    }

    fn lazy_sort(&mut self) {
        if self.is_dirty {
            self.items.sort();
            self.is_dirty = false;
        }
    }
}

#[derive(Debug)]
struct RankedItem<R, T, D>
where
    D: Direction,
    R: Ord,
{
    item: T,
    rank: R,
    seq: u64,
    _direction: std::marker::PhantomData<D>,
}

impl<R, T, D> RankedItem<R, T, D>
where
    D: Direction,
    R: Ord,
{
    fn new(item: T, rank: R, seq: u64) -> Self {
        Self {
            item,
            rank,
            seq,
            _direction: std::marker::PhantomData,
        }
    }
}

impl<R, T, D> Ord for RankedItem<R, T, D>
where
    D: Direction,
    R: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        D::cmp(&self.rank, self.seq, &other.rank, other.seq)
    }
}

impl<R, T, D> PartialOrd for RankedItem<R, T, D>
where
    D: Direction,
    R: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<R: Ord, T, D> Eq for RankedItem<R, T, D> where D: Direction {}

impl<R, T, D> PartialEq for RankedItem<R, T, D>
where
    D: Direction,
    R: Ord,
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

/// Marker type for [`RankedOrder`] values that yield the lowest-ranked items first.
pub struct Ascending;
/// Marker type for [`RankedOrder`] values that yield the highest-ranked items first.
pub struct Descending;

/// Strategy trait that defines how two ranked items are compared.
///
/// This is primarily intended for the built-in [`Ascending`] and [`Descending`]
/// marker types.
pub trait Direction {
    /// Compares two `(rank, insertion-sequence)` pairs using a concrete direction.
    fn cmp<R: Ord>(lhs_rank: &R, lhs_seq: u64, rhs_rank: &R, rhs_seq: u64) -> std::cmp::Ordering;
}
impl Direction for Ascending {
    fn cmp<R: Ord>(lhs_rank: &R, lhs_seq: u64, rhs_rank: &R, rhs_seq: u64) -> std::cmp::Ordering {
        rhs_rank.cmp(lhs_rank).then(rhs_seq.cmp(&lhs_seq))
    }
}
impl Direction for Descending {
    fn cmp<R: Ord>(lhs_rank: &R, lhs_seq: u64, rhs_rank: &R, rhs_seq: u64) -> std::cmp::Ordering {
        lhs_rank.cmp(rhs_rank).then(rhs_seq.cmp(&lhs_seq))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_ties_maintain_original_insertion_order() {
        let mut descender = DescendingOrder::new();
        descender.push('a', 1);
        descender.push('b', 1);
        descender.push('c', 1);

        assert_eq!(descender.pop(), Some(('a', 1)));
        assert_eq!(descender.pop(), Some(('b', 1)));
        assert_eq!(descender.pop(), Some(('c', 1)));

        let mut ascender = AscendingOrder::new();
        ascender.push('a', 1);
        ascender.push('b', 1);
        ascender.push('c', 1);

        assert_eq!(ascender.pop(), Some(('a', 1)));
        assert_eq!(ascender.pop(), Some(('b', 1)));
        assert_eq!(ascender.pop(), Some(('c', 1)));
    }

    #[test]
    fn iter_sorted_respects_direction() {
        let mut descender = DescendingOrder::new();
        descender.push('a', 1);
        descender.push('b', 2);
        descender.push('c', 3);

        assert_eq!(
            descender.iter_sorted().collect::<Vec<_>>(),
            vec![(&'c', &3), (&'b', &2), (&'a', &1)]
        );

        let mut ascender = AscendingOrder::new();
        ascender.push('a', 1);
        ascender.push('b', 2);
        ascender.push('c', 3);

        assert_eq!(
            ascender.iter_sorted().collect::<Vec<_>>(),
            vec![(&'a', &1), (&'b', &2), (&'c', &3)]
        );
    }

    #[test]
    fn into_sorted_vec_respects_direction() {
        let mut descender = DescendingOrder::new();
        descender.push('a', 1);
        descender.push('b', 2);
        descender.push('c', 3);

        assert_eq!(
            descender.into_sorted_vec(),
            vec![('c', 3), ('b', 2), ('a', 1)]
        );

        let mut ascender = AscendingOrder::new();
        ascender.push('a', 1);
        ascender.push('b', 2);
        ascender.push('c', 3);

        assert_eq!(
            ascender.into_sorted_vec(),
            vec![('a', 1), ('b', 2), ('c', 3)]
        );
    }

    #[test]
    fn push_with_ranker_uses_computed_rank() {
        let mut descender = DescendingOrder::new();
        descender.push_with_ranker("aaa", |item| item.len());
        descender.push_with_ranker("b", |item| item.len());
        descender.push_with_ranker("cc", |item| item.len());

        assert_eq!(
            descender.into_sorted_vec(),
            vec![("aaa", 3), ("cc", 2), ("b", 1)]
        );
    }

    #[test]
    fn pop_returns_none_when_empty() {
        let mut descender: DescendingOrder<i32, char> = DescendingOrder::new();
        let mut ascender: AscendingOrder<i32, char> = AscendingOrder::new();

        assert_eq!(descender.pop(), None);
        assert_eq!(ascender.pop(), None);
    }

    #[test]
    fn pushing_after_sorted_access_restores_ordering() {
        let mut descender = DescendingOrder::new();
        descender.push('a', 1);
        descender.push('b', 3);

        assert_eq!(
            descender.iter_sorted().collect::<Vec<_>>(),
            vec![(&'b', &3), (&'a', &1)]
        );

        descender.push('c', 2);

        assert_eq!(descender.pop(), Some(('b', 3)));
        assert_eq!(descender.pop(), Some(('c', 2)));
        assert_eq!(descender.pop(), Some(('a', 1)));
    }
}
