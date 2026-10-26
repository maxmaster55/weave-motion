//! Recorded traces of a spring simulation.

use weave_core::{Spring, SpringConfig};

/// What happened when a spring was retargeted mid-flight.
#[derive(Clone, Copy, Debug)]
pub struct RetargetEvent {
    /// When the retarget happened.
    #[allow(dead_code)]
    pub time: f64,
    /// The value right before the retarget.
    pub value_before: f64,
    /// The value right after the retarget.
    pub value_after: f64,
    /// The velocity right before the retarget.
    pub velocity_before: f64,
    /// The velocity right after the retarget.
    pub velocity_after: f64,
}

/// A recorded spring trajectory.
#[derive(Clone, Debug, Default)]
pub struct Trace {
    /// Timestamps of the samples.
    pub times: Vec<f64>,
    /// Sampled values.
    pub values: Vec<f64>,
    /// Sampled velocities.
    pub velocities: Vec<f64>,
    /// The time at which the spring settled, if it did.
    pub settled_at: Option<f64>,
    /// The retarget event, if the trace contains one.
    pub retarget: Option<RetargetEvent>,
}

impl Trace {
    /// The highest value in the trace.
    pub fn peak(&self) -> f64 {
        self.values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// The highest speed in the trace.
    pub fn peak_velocity(&self) -> f64 {
        self.velocities
            .iter()
            .copied()
            .fold(0.0_f64, |max, value| max.max(value.abs()))
    }

    /// How far the trace went past the target (never negative).
    pub fn overshoot(&self, target: f64) -> f64 {
        (self.peak() - target).max(0.0)
    }
}

/// Simulates a spring from `start` to `target` for `duration` seconds.
///
/// The simulation is driven by exact timestamps, so the result does not
/// depend on the frame rate — the playground can prove that by running the
/// same scenario at different rates.
pub fn simulate(
    config: SpringConfig,
    start: f64,
    target: f64,
    duration: f64,
    frame_rate: f64,
) -> Trace {
    let mut spring = Spring::new(start, target, config);
    let mut trace = Trace::default();

    let dt = 1.0 / frame_rate;
    let steps = (duration * frame_rate).ceil() as usize;

    for frame in 0..steps {
        let time = frame as f64 * dt;
        let active = spring.advance(dt);

        trace.times.push(time + dt);
        trace.values.push(spring.value());
        trace.velocities.push(spring.velocity());

        if !active {
            trace.settled_at = Some(time + dt);
            break;
        }
    }

    trace
}

/// Simulates a spring that is retargeted once, mid-flight.
///
/// The retarget keeps the current position **and** velocity: the trace must
/// stay continuous across the switch.
pub fn simulate_with_retarget(
    config: SpringConfig,
    start: f64,
    first_target: f64,
    second_target: f64,
    retarget_at: f64,
    duration: f64,
    frame_rate: f64,
) -> Trace {
    let mut spring = Spring::new(start, first_target, config);
    let mut trace = Trace::default();

    let dt = 1.0 / frame_rate;
    let steps = (duration * frame_rate).ceil() as usize;
    let mut retargeted = false;

    for frame in 0..steps {
        let time = frame as f64 * dt;

        if !retargeted && time >= retarget_at {
            let velocity_before = spring.velocity();
            let value_before = spring.value();
            spring.set_target(second_target);
            trace.retarget = Some(RetargetEvent {
                time,
                value_before,
                value_after: spring.value(),
                velocity_before,
                velocity_after: spring.velocity(),
            });
            retargeted = true;
        }

        let active = spring.advance(dt);

        trace.times.push(time + dt);
        trace.values.push(spring.value());
        trace.velocities.push(spring.velocity());

        if !active {
            trace.settled_at = Some(time + dt);
            break;
        }
    }

    trace
}

#[cfg(test)]
mod tests {
    use super::*;
    use weave_core::MotionToken;

    fn config() -> SpringConfig {
        MotionToken::DefaultSpatial.spring_config().unwrap()
    }

    #[test]
    fn a_trace_records_every_sample() {
        let trace = simulate(config(), 0.0, 1.0, 0.5, 60.0);
        assert_eq!(trace.times.len(), trace.values.len());
        assert_eq!(trace.values.len(), trace.velocities.len());
        assert!(trace.times.len() > 25);
    }

    #[test]
    fn a_trace_settles_at_its_target() {
        let trace = simulate(config(), 0.0, 1.0, 4.0, 120.0);
        assert!(trace.settled_at.is_some());
        let last = trace.values.last().copied().unwrap();
        assert!((last - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_retarget_changes_neither_the_value_nor_the_velocity() {
        let trace = simulate_with_retarget(config(), 0.0, 1.0, 0.25, 0.30, 2.0, 120.0);
        let event = trace.retarget.expect("the retarget happened");

        assert_eq!(
            event.value_before, event.value_after,
            "the position must survive the retarget"
        );
        assert_eq!(
            event.velocity_before, event.velocity_after,
            "the velocity must survive the retarget"
        );
    }

    #[test]
    fn a_retarget_keeps_the_trace_continuous() {
        let trace = simulate_with_retarget(config(), 0.0, 1.0, 0.25, 0.30, 2.0, 120.0);

        // No sample may move faster than the fastest velocity in the trace
        // allows: a jump would mean the animation restarted.
        let dt = 1.0 / 120.0;
        let allowed = trace.peak_velocity() * dt * 1.2;
        for pair in trace.values.windows(2) {
            let delta = (pair[1] - pair[0]).abs();
            assert!(delta <= allowed, "discontinuity of {delta} at {pair:?}");
        }
    }

    #[test]
    fn the_frame_rate_does_not_change_the_outcome() {
        let slow = simulate(config(), 0.0, 1.0, 1.0, 30.0);
        let fast = simulate(config(), 0.0, 1.0, 1.0, 120.0);

        let slow_last = slow.values.last().copied().unwrap();
        let fast_last = fast.values.last().copied().unwrap();
        assert!((slow_last - fast_last).abs() < 1e-6);
    }
}
