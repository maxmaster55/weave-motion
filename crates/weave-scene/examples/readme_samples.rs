//! Deterministic samples for the README illustrations; no renderer required.
use weaver_motion::{MotionToken, Spring};
use weave_geometry::{Radii, Rect, Shape};
use weave_scene::Scene;

fn main() {
    let tokens = [
        MotionToken::FastSpatial,
        MotionToken::DefaultSpatial,
        MotionToken::ExpressiveSpatial,
        MotionToken::GentleSpatial,
    ];
    let mut springs = tokens.map(|t| Spring::resting(0.0, t.spring_config().unwrap()));
    let mut retarget = Spring::resting(0.0, tokens[2].spring_config().unwrap());
    let mut scene = Scene::new();
    let pill = Rect::new(280.0, 125.0, 160.0, 56.0);
    let panel = Rect::new(140.0, 50.0, 440.0, 208.0);
    scene.layout("surface", pill);
    scene.radii("surface", Radii::uniform(28.0));
    for frame in 0..180 {
        if frame == 15 {
            for s in &mut springs {
                s.set_target(1.0);
            }
        }
        if frame == 95 {
            for s in &mut springs {
                s.set_target(0.0);
            }
        }
        match frame {
            15 | 105 => retarget.set_target(1.0),
            27 | 120 => retarget.set_target(0.0),
            _ => {}
        }
        if frame == 15 || frame == 100 {
            let bounds = if frame == 15 { panel } else { pill };
            scene.layout("surface", bounds);
            scene.radii(
                "surface",
                Radii::uniform(if frame == 15 { 24.0 } else { 28.0 }),
            );
        }
        for s in &mut springs {
            s.advance(1.0 / 30.0);
        }
        retarget.advance(1.0 / 30.0);
        scene.tick(frame as f64 / 30.0);
        let p = scene.presentation("surface").unwrap();
        let radius = match p.clip {
            Some(Shape::RoundedRect(r)) => r.radii.top_left,
            _ => 0.0,
        };
        println!(
            "{frame},{},{},{},{},{},{},{},{},{},{},{},{}",
            springs[0].value(),
            springs[1].value(),
            springs[2].value(),
            springs[3].value(),
            retarget.value(),
            retarget.target(),
            retarget.velocity(),
            p.bounds.x,
            p.bounds.y,
            p.bounds.width,
            p.bounds.height,
            radius
        );
    }
}
