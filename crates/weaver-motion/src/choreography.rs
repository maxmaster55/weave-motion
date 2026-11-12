//! Choreography helpers: stagger timing for groups that enter together.
//!
//! A stagger offsets the start of each item in a group. It is a budget, not
//! a style: perception research finds staggering does not help people track
//! moving objects (Chevalier, Dragicevic & Franconeri 2014), so the whole
//! cascade must stay short. The rule implemented here, from `docs/MOTION.md`:
//!
//! * interval = `min(30 ms, 500 ms / n)` for a group of `n` items
//!   (Carbon uses 20 ms within a 500 ms budget);
//! * at most [`STAGGER_MAX_ITEMS`] distinct start times: items beyond the
//!   eighth start with the eighth, so the spread is at most 7 intervals
//!   (210 ms).
//!
//! Use it only for first load and user-triggered bulk adds. Never stagger
//! re-sorts, routine live updates, menus or command palettes, and drop the
//! stagger (one group fade) under reduced motion.
//!
//! ```
//! use weaver_motion::stagger_delays;
//!
//! let delays = stagger_delays(10);
//! assert_eq!(delays.len(), 10);
//! assert_eq!(delays[0], 0.0);
//! assert!((delays[1] - 0.030).abs() < 1e-12);
//! // The ninth and tenth items start with the eighth.
//! assert_eq!(delays[9], delays[7]);
//! ```

use alloc::vec::Vec;

/// Longest interval between two staggered items, in seconds (30 ms).
pub const STAGGER_MAX_INTERVAL: f64 = 0.030;

/// Time budget the intervals of a whole group must fit, in seconds
/// (500 ms): with `n` items the interval is at most `STAGGER_BUDGET / n`.
pub const STAGGER_BUDGET: f64 = 0.500;

/// Most items that get their own start time. Later items start with the
/// last staggered one.
pub const STAGGER_MAX_ITEMS: usize = 8;

/// Interval between consecutive item start times for a group of `count`
/// items, in seconds: `min(30 ms, 500 ms / count)`. Zero for an empty group.
pub fn stagger_interval(count: usize) -> f64 {
    if count == 0 {
        return 0.0;
    }
    let even = STAGGER_BUDGET / count as f64;
    if even < STAGGER_MAX_INTERVAL {
        even
    } else {
        STAGGER_MAX_INTERVAL
    }
}

/// Start delay of item `index` (0-based) in a group of `count` items, in
/// seconds. Items at or beyond [`STAGGER_MAX_ITEMS`] share the last
/// staggered delay. Indices outside the group are treated like the last
/// item.
pub fn stagger_delay(index: usize, count: usize) -> f64 {
    if count == 0 {
        return 0.0;
    }
    let slot = index.min(count - 1).min(STAGGER_MAX_ITEMS - 1);
    slot as f64 * stagger_interval(count)
}

/// Start delays for every item of a group of `count` items, in seconds.
///
/// See [`stagger_delay`]. Pair each delay with
/// [`TweenConfig::with_delay`](crate::TweenConfig::with_delay), or start the
/// item's spring once its delay has elapsed.
pub fn stagger_delays(count: usize) -> Vec<f64> {
    (0..count)
        .map(|index| stagger_delay(index, count))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_is_capped_by_the_budget() {
        assert_eq!(stagger_interval(0), 0.0);
        assert_eq!(stagger_interval(1), STAGGER_MAX_INTERVAL);
        assert_eq!(stagger_interval(8), STAGGER_MAX_INTERVAL);
        assert_eq!(stagger_interval(16), STAGGER_MAX_INTERVAL);
        assert!((stagger_interval(20) - 0.025).abs() < 1e-12);
        assert!((stagger_interval(100) - 0.005).abs() < 1e-12);
    }

    #[test]
    fn at_most_eight_distinct_start_times() {
        for count in [1, 2, 7, 8, 9, 12, 40, 500] {
            let delays = stagger_delays(count);
            assert_eq!(delays.len(), count);
            assert_eq!(delays[0], 0.0);
            let mut distinct = delays.clone();
            distinct.dedup();
            assert_eq!(distinct.len(), count.min(STAGGER_MAX_ITEMS), "{count}");
            assert!(delays.windows(2).all(|w| w[0] <= w[1]));
            let spread = *delays.last().unwrap();
            assert!(spread <= 7.0 * STAGGER_MAX_INTERVAL + 1e-12, "{count}");
            assert!(spread < STAGGER_BUDGET);
        }
    }

    #[test]
    fn items_beyond_the_cap_start_with_the_last() {
        let delays = stagger_delays(12);
        for delay in &delays[STAGGER_MAX_ITEMS..] {
            assert_eq!(*delay, delays[STAGGER_MAX_ITEMS - 1]);
        }
        assert!((delays[7] - 0.210).abs() < 1e-12);
    }

    #[test]
    fn out_of_range_indices_are_clamped() {
        assert_eq!(stagger_delay(3, 0), 0.0);
        assert_eq!(stagger_delay(5, 3), stagger_delay(2, 3));
    }
}
