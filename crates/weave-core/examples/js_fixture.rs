//! Reference trajectories for the JavaScript port (`js/weave.js`).
//!
//! `cargo run -q -p weave-core --example js_fixture > js/test/fixtures/springs.json`
//! The JS test suite replays the same cases and compares every sample.
use weave_core::{solve, MotionPolicy, MotionToken, Spring};

fn main() {
    let tokens = [
        ("fast", MotionToken::FastSpatial),
        ("default", MotionToken::DefaultSpatial),
        ("expressive", MotionToken::ExpressiveSpatial),
        ("gentle", MotionToken::GentleSpatial),
    ];
    let policies = [
        ("full", MotionPolicy::Full),
        ("reduced", MotionPolicy::Reduced),
        ("off", MotionPolicy::Off),
    ];
    // (start value, start velocity, target)
    let starts = [(0.0, 0.0, 1.0), (100.0, -250.0, -40.0), (-3.5, 12.0, 7.25)];
    let mut cases = Vec::new();
    for (name, token) in tokens {
        for (pname, policy) in policies {
            let config = policy.spring_config(token.spring_config().unwrap());
            for (value, velocity, target) in starts {
                let mut spring = Spring::with_velocity(value, velocity, target, config);
                let mut samples = Vec::new();
                for _ in 0..90 {
                    let moving = spring.advance(1.0 / 60.0);
                    samples.push(format!(
                        "[{:?},{:?},{}]",
                        spring.value(),
                        spring.velocity(),
                        moving
                    ));
                }
                // One large step: the exact-time solver must not depend on the step count.
                let (big_v, big_vel) = solve(&config, value, velocity, target, 0.75);
                cases.push(format!(
                    "{{\"preset\":\"{name}\",\"policy\":\"{pname}\",\"value\":{value:?},\"velocity\":{velocity:?},\"target\":{target:?},\"samples\":[{}],\"big\":[{big_v:?},{big_vel:?}]}}",
                    samples.join(",")
                ));
            }
        }
    }
    println!("[\n{}\n]", cases.join(",\n"));
}
