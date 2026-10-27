//! The shared monotonic clock.
//!
//! Weave never reads the wall clock. Every animation is driven by
//! timestamps the host application provides — ideally the ones the renderer
//! already uses for its frame callbacks. The clock turns those timestamps
//! into elapsed seconds and decides what happens when a long gap appears
//! (an application that was suspended, a debugger that was paused, a
//! window that was occluded).

/// What to do when the time between two frames is unexpectedly large.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeGapPolicy {
    /// Simulate the whole gap analytically. This is the default: springs
    /// are solved in closed form, so a one second gap costs exactly as much
    /// as a one second step.
    #[default]
    FastForward,
    /// Ignore the gap entirely. Useful for applications that were
    /// backgrounded and should look as if they were frozen.
    Freeze,
    /// Ignore the gap and settle every animation onto its destination.
    Snap,
}

/// The result of advancing the clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tick {
    /// Seconds elapsed since the previous tick.
    pub dt: f64,
    /// `true` when the gap policy asked for the animations to snap.
    pub snapped: bool,
}

/// A monotonic clock shared by every animation of an application.
///
/// ```
/// use weaver_motion::MotionClock;
///
/// let mut clock = MotionClock::new();
/// assert_eq!(clock.tick(0.0).dt, 0.0, "the first tick has no predecessor");
/// assert_eq!(clock.tick(1.0 / 60.0).dt, 1.0 / 60.0);
/// ```
#[derive(Clone, Debug, Default)]
pub struct MotionClock {
    last: Option<f64>,
    gap_policy: TimeGapPolicy,
    paused: bool,
    /// Optional threshold above which a gap is considered "large".
    gap_threshold: Option<f64>,
}

impl MotionClock {
    /// Creates a clock with the default [`TimeGapPolicy::FastForward`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a clock with an explicit gap policy.
    pub fn with_gap_policy(gap_policy: TimeGapPolicy) -> Self {
        Self {
            gap_policy,
            ..Self::default()
        }
    }

    /// The current gap policy.
    pub fn gap_policy(&self) -> TimeGapPolicy {
        self.gap_policy
    }

    /// Replaces the gap policy.
    pub fn set_gap_policy(&mut self, gap_policy: TimeGapPolicy) {
        self.gap_policy = gap_policy;
    }

    /// Sets the threshold above which a gap counts as "large".
    ///
    /// `None` (the default) treats every gap as normal, which is what the
    /// analytic solver wants.
    pub fn set_gap_threshold(&mut self, threshold: Option<f64>) {
        self.gap_threshold = threshold;
    }

    /// Pauses the clock: ticks report a zero delta until it is resumed.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the clock.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Whether the clock is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Forgets the last timestamp, so the next tick starts a new interval.
    pub fn reset(&mut self) {
        self.last = None;
    }

    /// Advances the clock and returns the elapsed time.
    ///
    /// Timestamps must be monotonic. A timestamp that goes backwards (a
    /// clock reset, a broken driver) is ignored and reported as a zero
    /// delta instead of producing a negative one.
    pub fn tick(&mut self, timestamp: f64) -> Tick {
        if !timestamp.is_finite() {
            return Tick {
                dt: 0.0,
                snapped: false,
            };
        }

        let previous = match self.last {
            Some(previous) => previous,
            None => {
                self.last = Some(timestamp);
                return Tick {
                    dt: 0.0,
                    snapped: false,
                };
            }
        };

        self.last = Some(timestamp);

        if self.paused {
            return Tick {
                dt: 0.0,
                snapped: false,
            };
        }

        if timestamp <= previous {
            // Not monotonic: ignore instead of running the simulation
            // backwards.
            return Tick {
                dt: 0.0,
                snapped: false,
            };
        }

        let raw = timestamp - previous;
        let is_large_gap = self
            .gap_threshold
            .map(|threshold| raw > threshold)
            .unwrap_or(false);

        if is_large_gap {
            match self.gap_policy {
                TimeGapPolicy::FastForward => Tick {
                    dt: raw,
                    snapped: false,
                },
                TimeGapPolicy::Freeze => Tick {
                    dt: 0.0,
                    snapped: false,
                },
                TimeGapPolicy::Snap => Tick {
                    dt: 0.0,
                    snapped: true,
                },
            }
        } else {
            Tick {
                dt: raw,
                snapped: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_tick_has_no_delta() {
        let mut clock = MotionClock::new();
        assert_eq!(clock.tick(1.0).dt, 0.0);
        assert_eq!(clock.tick(1.5).dt, 0.5);
    }

    #[test]
    fn backwards_timestamps_are_ignored() {
        let mut clock = MotionClock::new();
        clock.tick(1.0);
        let tick = clock.tick(0.5);
        assert_eq!(tick.dt, 0.0);
    }

    #[test]
    fn non_finite_timestamps_are_ignored() {
        let mut clock = MotionClock::new();
        clock.tick(1.0);
        assert_eq!(clock.tick(f64::NAN).dt, 0.0);
        assert_eq!(clock.tick(f64::INFINITY).dt, 0.0);
    }

    #[test]
    fn pause_freezes_time() {
        let mut clock = MotionClock::new();
        clock.tick(0.0);
        clock.pause();
        assert_eq!(clock.tick(1.0).dt, 0.0);
        clock.resume();
        assert_eq!(clock.tick(1.5).dt, 0.5);
    }

    #[test]
    fn gap_policies_differ_on_large_gaps() {
        let mut clock = MotionClock::new();
        clock.set_gap_threshold(Some(0.5));
        clock.tick(0.0);

        clock.set_gap_policy(TimeGapPolicy::FastForward);
        assert_eq!(clock.tick(10.0).dt, 10.0);

        clock.tick(10.0);
        clock.set_gap_policy(TimeGapPolicy::Freeze);
        let tick = clock.tick(20.0);
        assert_eq!(tick.dt, 0.0);
        assert!(!tick.snapped);

        clock.tick(20.0);
        clock.set_gap_policy(TimeGapPolicy::Snap);
        let tick = clock.tick(30.0);
        assert_eq!(tick.dt, 0.0);
        assert!(tick.snapped);
    }

    #[test]
    fn reset_forgets_the_previous_timestamp() {
        let mut clock = MotionClock::new();
        clock.tick(5.0);
        clock.reset();
        assert_eq!(clock.tick(6.0).dt, 0.0);
        assert_eq!(clock.tick(6.5).dt, 0.5);
    }
}
