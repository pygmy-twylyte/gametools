//! `ordering` - types for handling anything in a particular order.
//!
//! While some of these types appear to do essentially the same job, the underlying
//! mechanics can make a difference in terms of which is best to use:
//!
//! ## `RankedOrder` is best when:
//! - you need the full ranked order, not just the #1 priority
//! - stable visible ordering (as in UI display) matters
//! - queue sizes are small to moderate
//! - changing priorities of items already in place may be required
//! - high throughput is not a big concern
//!
//! ## `PriorityQueue` is best when:
//! - you just need quick access to the highest ranked item in a pool
//! - you rarely need to inspect the order beyond the element with top priority
//! - queue sizes are large and are modified frequently
//!

pub mod ranked_order;
pub use ranked_order::*;

pub mod priority_queue;
pub use priority_queue::*;
