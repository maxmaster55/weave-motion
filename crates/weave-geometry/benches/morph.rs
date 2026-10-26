//! A dependency free benchmark of the geometry layer.
//!
//! Run it with:
//!
//! ```text
//! cargo bench -p weave-geometry
//! ```

use std::hint::black_box;
use std::time::Instant;

use weave_geometry::{MorphPlan, Path, Point, Rect, RoundedRect};

const FRAMES: usize = 600;

fn circle(radius: f64, samples: usize) -> Path {
    let points: Vec<Point> = (0..samples)
        .map(|index| {
            let angle = index as f64 / samples as f64 * core::f64::consts::TAU;
            Point::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect();
    Path::from_points(&points, true)
}

fn hexagon(radius: f64) -> Path {
    let points: Vec<Point> = (0..6)
        .map(|index| {
            let angle = index as f64 / 6.0 * core::f64::consts::TAU;
            Point::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect();
    Path::from_points(&points, true)
}

fn measure_plan() {
    let from = circle(50.0, 64);
    let to = hexagon(50.0);

    let start = Instant::now();
    let mut builds = 0;
    while builds < 100 {
        builds += 1;
        black_box(MorphPlan::build(&from, &to));
    }
    let elapsed = start.elapsed();

    println!(
        "{:<28} {:>8.3} ms/plan (correspondence: resample, align, validate)",
        "morph plan",
        elapsed.as_secs_f64() * 1000.0 / builds as f64
    );
}

fn measure_evaluation() {
    let from = circle(50.0, 64);
    let to = hexagon(50.0);
    let plan = MorphPlan::build(&from, &to).expect("a valid plan");

    let bounds = Rect::new(0.0, 0.0, 400.0, 300.0);

    let start = Instant::now();
    for frame in 0..FRAMES {
        let progress = (frame % 60) as f64 / 60.0;
        black_box(plan.path_in(progress, bounds));
    }
    let elapsed = start.elapsed();

    println!(
        "{:<28} {:>8.4} ms/frame ({:>6} points)",
        "morph evaluation",
        elapsed.as_secs_f64() * 1000.0 / FRAMES as f64,
        plan.points(0.5).len()
    );
}

fn measure_rounded_rect() {
    let from = RoundedRect::pill(Rect::new(0.0, 0.0, 120.0, 40.0));
    let to = RoundedRect::uniform(Rect::new(0.0, 0.0, 400.0, 300.0), 24.0);

    let start = Instant::now();
    for frame in 0..FRAMES {
        let progress = (frame % 60) as f64 / 60.0;
        black_box(from.lerp(&to, progress));
    }
    let elapsed = start.elapsed();

    println!(
        "{:<28} {:>8.5} ms/frame",
        "rounded rect interpolation",
        elapsed.as_secs_f64() * 1000.0 / FRAMES as f64
    );
}

fn main() {
    println!("Weave geometry benchmarks");
    println!("========================\n");

    measure_rounded_rect();
    measure_plan();
    measure_evaluation();
}
