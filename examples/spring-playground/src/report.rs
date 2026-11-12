//! The scripted part of the playground: a report that works everywhere.
//!
//! Running `cargo run -p spring-playground` prints this report. It needs
//! no terminal, no input and no GPU, which makes it usable from CI and
//! from a build machine.

use weave_demo_support::Canvas;
use weaver_motion::{MotionPolicy, MotionToken};

use crate::trace::{simulate, simulate_with_retarget, Trace};

const CHART_WIDTH: usize = 72;
const CHART_HEIGHT: usize = 16;

/// Renders a chart of `trace` as text.
fn render_chart(title: &str, trace: &Trace, minimum: f64, maximum: f64, target: f64) -> String {
    let mut canvas = Canvas::new(CHART_WIDTH, CHART_HEIGHT);
    canvas.text(0, -1, title);

    // Frame: a box around the plot area.
    canvas.hline(0, 0, CHART_WIDTH as isize - 1, '-');
    canvas.hline(CHART_HEIGHT as isize - 1, 0, CHART_WIDTH as isize - 1, '-');
    canvas.vline(0, 0, CHART_HEIGHT as isize - 1, '|');
    canvas.vline(CHART_WIDTH as isize - 1, 0, CHART_HEIGHT as isize - 1, '|');

    // Grid lines at 25%, 50% and 75% of the range.
    for step in 1..4 {
        let y = (CHART_HEIGHT as f64 * step as f64 / 4.0).round() as isize;
        canvas.hline(y, 1, CHART_WIDTH as isize - 2, '.');
    }

    let plot_width = CHART_WIDTH - 2;
    for (index, value) in trace.values.iter().enumerate() {
        if index >= plot_width {
            break;
        }
        let normalized = ((value - minimum) / (maximum - minimum)).clamp(0.0, 1.0);
        let y = (CHART_HEIGHT as f64 - 2.0) * (1.0 - normalized);
        let y = 1 + y.round() as isize;
        canvas.set(1 + index as isize, y, '*');
    }

    // Target marker.
    let target_normalized = ((target - minimum) / (maximum - minimum)).clamp(0.0, 1.0);
    let y = (CHART_HEIGHT as f64 - 2.0) * (1.0 - target_normalized);
    let y = 1 + y.round() as isize;
    canvas.hline(y, 1, CHART_WIDTH as isize - 2, '-');

    format!("{}\n{}", title, canvas.render())
}

/// Prints the report.
pub fn run() {
    println!("Weave spring playground");
    println!("=======================\n");

    preset_comparison();
    println!();
    trajectory();
    println!();
    interruption();
    println!();
    frame_rate_independence();
    println!();
    reduced_motion();
}

/// Scenario 1: how the presets behave.
fn preset_comparison() {
    println!("Presets");
    println!("-------");
    println!(
        "{:<18} {:>10} {:>12} {:>12}",
        "token", "settle", "overshoot", "peak speed"
    );

    for token in MotionToken::ALL {
        let Some(config) = token.spring_config() else {
            continue;
        };
        let trace = simulate(config, 0.0, 1.0, 4.0, 120.0);
        let settle = trace
            .settled_at
            .map(|seconds| format!("{seconds:.3}s"))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<18} {:>10} {:>11.2}% {:>11.2}/s",
            token.name(),
            settle,
            trace.overshoot(1.0) * 100.0,
            trace.peak_velocity()
        );
    }
}

/// Scenario 2: the trajectory of the default preset.
fn trajectory() {
    let config = MotionToken::DefaultSpatial.spring_config().unwrap();
    let trace = simulate(config, 0.0, 1.0, 2.0, 120.0);

    println!(
        "{}",
        render_chart("default_spatial: 0 -> 1", &trace, 0.0, 1.3, 1.0)
    );
}

/// Scenario 3: retargeting mid-flight keeps the motion continuous.
fn interruption() {
    let config = MotionToken::ExpressiveSpatial.spring_config().unwrap();
    let trace = simulate_with_retarget(config, 0.0, 1.0, 0.25, 0.30, 2.0, 120.0);

    println!(
        "{}",
        render_chart(
            "expressive_spatial: retarget 1.0 -> 0.25 at t=0.30s",
            &trace,
            -0.1,
            1.3,
            0.25,
        )
    );

    // Show the retarget itself: neither the value nor the velocity may
    // change when the destination changes.
    if let Some(event) = trace.retarget {
        println!("  the retarget itself (t={:.4}s):", event.time);
        println!(
            "    before: value={:.5}  velocity={:+.5}",
            event.value_before, event.velocity_before
        );
        println!(
            "    after : value={:.5}  velocity={:+.5}",
            event.value_after, event.velocity_after
        );
    }

    // Show the frames around the retarget to make the continuity visible.
    let dt = 1.0 / 120.0;
    let around = (0.30 / dt) as usize;
    println!("  frames around the retarget:");
    for frame in (around.saturating_sub(2))..=(around + 2) {
        let index = frame.min(trace.values.len().saturating_sub(1));
        let marker = if trace.times[index] >= 0.30 {
            "  <- retarget"
        } else {
            ""
        };
        println!(
            "    t={:.4}s  value={:.5}  velocity={:+.5}{}",
            trace.times[index], trace.values[index], trace.velocities[index], marker
        );
    }
}

/// Scenario 4: the result does not depend on the frame rate.
fn frame_rate_independence() {
    println!("Frame rate independence (same timestamps, different step counts)");
    println!("    (0 -> 1 over 0.4 s)");
    let config = MotionToken::DefaultSpatial.spring_config().unwrap();

    for frame_rate in [30.0, 60.0, 90.0, 120.0] {
        let trace = simulate(config, 0.0, 1.0, 0.4, frame_rate);
        let value = trace.values.last().copied().unwrap_or_default();
        let velocity = trace.velocities.last().copied().unwrap_or_default();
        println!("    {frame_rate:>5.0} Hz -> value={value:.10}  velocity={velocity:+.10}");
    }
}

/// Scenario 5: reduced motion removes the overshoot.
fn reduced_motion() {
    println!("Reduced motion");
    let config = MotionToken::ExpressiveSpatial.spring_config().unwrap();
    let reduced = MotionPolicy::Reduced.spring_config(config);

    let full = simulate(config, 0.0, 1.0, 2.0, 120.0);
    let gentle = simulate(reduced, 0.0, 1.0, 2.0, 120.0);

    println!(
        "    full     overshoot={:.2}%  settle={:?}",
        full.overshoot(1.0) * 100.0,
        full.settled_at
    );
    println!(
        "    reduced  overshoot={:.2}%  settle={:?}",
        gentle.overshoot(1.0) * 100.0,
        gentle.settled_at
    );
}
