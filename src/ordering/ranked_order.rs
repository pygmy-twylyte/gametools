use std::marker::PhantomData;

pub type DescendingOrder<R, T> = RankedOrder<R, T, Descending>;
pub type AscendingOrder<R, T> = RankedOrder<R, T, Ascending>;

pub struct RankedOrder<R: Ord, T, D>
where
    D: Direction,
{
    items: Vec<RankedItem<R, T, D>>,
    seq: u64,
    is_dirty: bool,
}

impl<R: Ord, T, D> RankedOrder<R, T, D>
where
    D: Direction,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            seq: 0,
            is_dirty: false,
        }
    }

    fn push_inner(&mut self, item: T, rank: R)
    where
        D: Direction,
    {
        self.seq = self.seq.wrapping_add(1);
        self.items
            .push(RankedItem::new(item, rank, self.seq, PhantomData));
        self.is_dirty = true;
    }

    fn pop_inner(&mut self) -> Option<(T, R)> {
        if self.is_dirty {
            self.items.sort()
        }
        self.items.pop().map(|ri| (ri.item, ri.rank))
    }
}

impl<R: Ord, T> RankedOrder<R, T, Descending> {
    pub fn push(&mut self, item: T, rank: R) {
        self.push_inner(item, rank);
    }

    pub fn pop(&mut self) -> Option<(T, R)> {
        self.pop_inner()
    }
}

impl<R: Ord, T> RankedOrder<R, T, Ascending> {
    pub fn push(&mut self, item: T, rank: R) {
        self.push_inner(item, rank);
    }

    pub fn pop(&mut self) -> Option<(T, R)> {
        self.pop_inner()
    }
}

struct RankedItem<R: Ord, T, D>
where
    D: Direction,
{
    item: T,
    rank: R,
    seq: u64,
    _direction: std::marker::PhantomData<D>,
}

impl<R: Ord, T, D> RankedItem<R, T, D>
where
    D: Direction,
{
    fn new(item: T, rank: R, seq: u64, _direction: std::marker::PhantomData<D>) -> Self {
        Self {
            item,
            rank,
            seq,
            _direction: std::marker::PhantomData,
        }
    }
}
impl<R: Ord, T, D> Ord for RankedItem<R, T, D>
where
    D: Direction,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        D::cmp(&self.rank, self.seq, &other.rank, other.seq)
    }
}
impl<R: Ord, T, D> PartialOrd for RankedItem<R, T, D>
where
    D: Direction,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<R: Ord, T, D> Eq for RankedItem<R, T, D> where D: Direction {}
impl<R: Ord, T, D> PartialEq for RankedItem<R, T, D>
where
    D: Direction,
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
        lhs_rank.cmp(rhs_rank).then(lhs_seq.cmp(&rhs_seq))
    }
}
