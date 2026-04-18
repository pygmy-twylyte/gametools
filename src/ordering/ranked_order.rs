pub type AscendingOrder<R, T> = RankedOrder<R, T, Ascending>;
pub type DescendingOrder<R, T> = RankedOrder<R, T, Descending>;

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
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            seq: 0,
            is_dirty: false,
        }
    }

    pub fn push(&mut self, item: T, rank: R) {
        self.push_inner(item, rank);
    }

    pub fn push_with_ranker<F: FnOnce(&T) -> R>(&mut self, item: T, ranker: F) {
        let rank = ranker(&item);
        self.push_inner(item, rank);
    }

    pub fn pop(&mut self) -> Option<(T, R)> {
        self.pop_inner()
    }

    pub fn into_sorted_vec(mut self) -> Vec<(T, R)> {
        self.lazy_sort();
        self.items
            .into_iter()
            .rev()
            .map(|ri| (ri.item, ri.rank))
            .collect()
    }

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

pub struct Ascending;
pub struct Descending;

pub trait Direction {
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
}
