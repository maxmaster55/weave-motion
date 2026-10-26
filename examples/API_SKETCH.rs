//! Non-compiling illustrative usage: shows an intended public API direction.
//! APIs below DO NOT exist until implemented.

use weave::{MotionEngine, MotionPreset, Shape};

fn illustrative_usage(mut engine: MotionEngine) {
    let button = engine.spring("create_button.scale", 1.0, MotionPreset::FastSpatial);
    button.set_target(1.06); // Do not reset velocity.

    engine.morph(
        "create_button.outline",
        Shape::rounded_rect(12.0),
        Shape::rounded_rect(32.0),
        MotionPreset::ExpressiveSpatial,
    );

    engine.shared_transition("create_button", /* source */ (), /* target */ ());
    let _snapshot = engine.tick(/* monotonic_timestamp */ 0.0);
}
