//! A dependency free benchmark of the spring solver and the scheduler.
//!
//! Run it with:
//!
//! ```text
//! cargo bench -p weaver-motion
//! ```
//!
//! The numbers are printed in a form that can be pasted into
//! `docs/PROFILING.md`.

use std::hint::black_box;
use std::time::Instant;

use weaver_motion::{Animated, MotionToken, Scheduler, Spring, Vec2};

const FRAMES: usize = 600;

fn measure(label: &str, count: usize, budget_ms: f64) {
    let config = MotionToken::DefaultSpatial
        .spring_config()
        .expect("spatial token");
    let mut springs: Vec<Animated<f64>> = (0..count)
        .map(|_| Animated::new(0.0, 1.0, config))
        .collect();

    // Warm up.
    for spring in springs.iter_mut() {
        spring.advance(1.0 / 60.0);
    }

    let start = Instant::now();
    let mut frames = 0;
    while frames < FRAMES {
        frames += 1;
        // Keep the springs in motion: a settled spring does no work at all.
        if frames % 30 == 0 {
            let target = if (frames / 30) % 2 == 0 { 1.0 } else { 0.0 };
            for spring in springs.iter_mut() {
                spring.set_target(target);
            }
        }
        for spring in springs.iter_mut() {
            spring.advance(1.0 / 60.0);
            black_box(spring.value());
        }
    }
    let elapsed = start.elapsed();

    let per_frame = elapsed.as_secs_f64() * 1000.0 / FRAMES as f64;
    let per_spring = elapsed.as_secs_f64() * 1e9 / (FRAMES * count) as f64;
    let share = per_frame / budget_ms * 100.0;

    println!(
        "{label:<28} {count:>6} springs  {per_frame:>8.4} ms/frame  {per_spring:>7.1} ns/spring  \
         {share:>5.2}% of a {budget_ms:.1} ms budget",
    );
}

fn measure_scheduler(count: usize) {
    let mut scheduler = Scheduler::new();
    let config = MotionToken::DefaultSpatial
        .spring_config()
        .expect("spatial token");

    for index in 0..count {
        scheduler.insert(
            Some(format!("spring.{index}")),
            Box::new(Animated::new(0.0, 1.0, config)),
        );
    }

    let start = Instant::now();
    let mut frames = 0;
    while frames < FRAMES {
        frames += 1;
        scheduler.tick(frames as f64 / 60.0);
    }
    let elapsed = start.elapsed();

    let per_frame = elapsed.as_secs_f64() * 1000.0 / FRAMES as f64;
    println!(
        "{:<28} {count:>6} springs  {per_frame:>8.4} ms/frame (scheduler + snapshot)",
        "scheduler"
    );
}

fn measure_single() {
    let config = MotionToken::DefaultSpatial
        .spring_config()
        .expect("spatial token");
    let mut spring = Spring::new(0.0, 1.0, config);

    let start = Instant::now();
    for step in 0..1_000_000 {
        if step % 60 == 0 {
            spring.set_target(if (step / 60) % 2 == 0 { 1.0 } else { 0.0 });
        }
        spring.advance(1.0 / 60.0);
        black_box(spring.value());
    }
    let elapsed = start.elapsed();

    println!(
        "{:<28} {:>6} ns/update (analytic solver)",
        "single spring",
        elapsed.as_secs_f64() * 1e9 / 1_000_000.0
    );
}

fn measure_vector() {
    let config = MotionToken::DefaultSpatial
        .spring_config()
        .expect("spatial token");
    let mut animated = Animated::new(Vec2::ZERO, Vec2::splat(1.0), config);

    let start = Instant::now();
    for step in 0..1_000_000 {
        if step % 60 == 0 {
            animated.set_target(if (step / 60) % 2 == 0 {
                Vec2::splat(1.0)
            } else {
                Vec2::ZERO
            });
        }
        animated.advance(1.0 / 60.0);
        black_box(animated.value());
    }
    let elapsed = start.elapsed();

    println!(
        "{:<28} {:>6} ns/update (two components)",
        "vector spring",
        elapsed.as_secs_f64() * 1e9 / 1_000_000.0
    );
}

fn main() {
    println!("Weave core benchmarks");
    println!("=====================\n");

    measure_single();
    measure_vector();
    println!();

    measure("springs (60 Hz)", 100, 16.67);
    measure("springs (60 Hz)", 1_000, 16.67);
    measure("springs (60 Hz)", 10_000, 16.67);
    println!();

    measure_scheduler(100);
    measure_scheduler(1_000);
}
