//! The scripted acceptance run of the pill-to-panel demo.
//!
//! This is the "first demo acceptance test" of the design: a rounded button
//! expands into a panel and reverses at an arbitrary point during the
//! movement; position and velocity stay continuous, the clipping stays
//! correct, the text stays readable and the animation goes idle on settle.
//!
//! It runs without a terminal, without input and without a GPU, so it can
//! be part of a build.

use weaver_motion::MotionPolicy;

use crate::surface::{Demo, ICON, LABEL, SURFACE};

const FRAME: f64 = 1.0 / 60.0;

/// Runs the acceptance scenario and prints a report.
pub fn run() {
    println!("Weave button-to-panel demo");
    println!("==========================\n");

    let mut demo = Demo::new();
    demo.tick(0.0);

    println!("state: {}", demo.status());
    println!("   expanded: {}", demo.is_expanded());

    // ------------------------------------------------------------------
    // 1. Expansion
    // ------------------------------------------------------------------
    println!("\n1. expansion");
    demo.set_expanded(true);

    let mut frames = 0;
    let mut settle_time = None;
    let mut overshoot = 0.0_f64;
    let mut revealed_at = None;
    let mut icon_hidden_at = None;

    while frames < 600 {
        frames += 1;
        demo.tick(frames as f64 * FRAME);

        let bounds = demo.scene().animated_bounds(SURFACE).expect("bounds");
        overshoot = overshoot.max(bounds.width - crate::surface::PANEL.width);

        if revealed_at.is_none() {
            if let Some(label) = demo.scene().presentation(LABEL) {
                if label.content_visible {
                    revealed_at = Some(frames);
                }
            }
        }
        if icon_hidden_at.is_none() {
            if let Some(icon) = demo.scene().presentation(ICON) {
                if !icon.content_visible {
                    icon_hidden_at = Some(frames);
                }
            }
        }

        if demo.scene().is_idle() {
            settle_time = Some(frames as f64 * FRAME);
            break;
        }
    }

    println!(
        "   settled after {:.3}s ({} frames), idle: {}",
        settle_time.unwrap_or_default(),
        frames,
        demo.scene().is_idle()
    );
    println!(
        "   label revealed after {} frames, icon hidden after {} frames",
        revealed_at
            .map(|f| f.to_string())
            .unwrap_or_else(|| "-".into()),
        icon_hidden_at
            .map(|f| f.to_string())
            .unwrap_or_else(|| "-".into()),
    );
    println!("   horizontal overshoot: {:.4} cells", overshoot.max(0.0));

    // ------------------------------------------------------------------
    // 2. Interruption in mid-flight
    // ------------------------------------------------------------------
    println!("\n2. interruption in mid-flight");
    demo.set_expanded(false);
    for frame in 0..12 {
        demo.tick((frames + frame) as f64 * FRAME);
    }

    let before = demo
        .scene()
        .animated_bounds(SURFACE)
        .expect("bounds before the interruption");
    let velocity_before = demo
        .scene()
        .bounds_velocity(SURFACE)
        .expect("velocity before the interruption");

    // Reverse while the surface is still moving.
    demo.set_expanded(true);
    let after = demo
        .scene()
        .animated_bounds(SURFACE)
        .expect("bounds after the interruption");
    let velocity_after = demo
        .scene()
        .bounds_velocity(SURFACE)
        .expect("velocity after the interruption");

    println!(
        "   bounds   {:.3} -> {:.3} (jump {:.6})",
        before.width,
        after.width,
        (after.width - before.width).abs()
    );
    println!(
        "   velocity {:.3} -> {:.3} (jump {:.6})",
        velocity_before.width,
        velocity_after.width,
        (velocity_after.width - velocity_before.width).abs()
    );

    // ------------------------------------------------------------------
    // 3. Settling
    // ------------------------------------------------------------------
    println!("\n3. settling");
    let mut frames = 0;
    while frames < 600 {
        frames += 1;
        demo.tick((600 + frames) as f64 * FRAME);
        if demo.scene().is_idle() {
            break;
        }
    }
    println!(
        "   idle after {} frames, needs_frame: {}",
        frames,
        demo.scene().needs_frame()
    );
    println!("   state: {}", demo.status());

    // ------------------------------------------------------------------
    // 4. Reduced motion
    // ------------------------------------------------------------------
    println!("\n4. reduced motion");
    let mut reduced = Demo::with_policy(MotionPolicy::Reduced);
    reduced.tick(0.0);
    reduced.set_expanded(true);

    let mut frames = 0;
    let mut overshoot = 0.0_f64;
    while frames < 600 {
        frames += 1;
        reduced.tick(frames as f64 * FRAME);
        let bounds = reduced.scene().animated_bounds(SURFACE).expect("bounds");
        overshoot = overshoot.max(bounds.width - crate::surface::PANEL.width);
        if reduced.scene().is_idle() {
            break;
        }
    }
    println!(
        "   settled after {} frames, overshoot: {:.4} cells",
        frames,
        overshoot.max(0.0)
    );
    println!("   state: {}", reduced.status());
}
