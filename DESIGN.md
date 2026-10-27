# Weave: Technical Design Specification

**Version:** 0.1 (design proposal)  
**Language:** Rust  
**Purpose:** reusable expressive physics and morphing motion for UI projects  
**Relationship:** independent engine usable by Weaver, Loom and other applications

## 1. Vision

Weave animates *relationships and state changes*, not just isolated CSS-like properties. It aims for a responsive, fluid, physical feel inspired by Material 3 Expressive: smooth spatial continuity, velocity-preserving springs, transformable outlines, and coordinated but restrained expressive feedback.

Weave is **not** a new window system, layout engine, widget toolkit, GPU renderer, or clone of Google's Material library. Its output is a presentation-state snapshot consumed by adapters.

**Principles**

1. **Retargetable:** a new target never snaps a moving value or discards its velocity by default.
2. **Identity-preserving:** UI entities retain stable logical identity across layout and navigation changes.
3. **Continuous geometry:** compatible shapes interpolate rather than abruptly replace one another.
4. **State-driven:** apps specify destination state; Weave derives animated presentation state.
5. **Composable:** core physics has no dependencies on wgpu, Compose, DOM, or UI framework internals.
6. **Accessible:** reduced motion preserves state information while minimizing unnecessary displacement.
7. **Efficient:** animations stop requesting frames when at rest; cached geometric work stays cached.
8. **Semantic:** motion clarifies relationships, intent, hierarchy and navigation rather than decorating everything.

## 2. Relationship to Material 3 Expressive

The inspiration includes spring-driven spatial motion, separate effects motion, shape morphing and shared-element transitions. Weave should not describe its presets as official Material numeric tokens. Official APIs differ by platform and version. See [Google's MotionScheme docs](https://developer.android.com/reference/kotlin/androidx/compose/material3/MotionScheme), [Compose shapes](https://developer.android.com/develop/ui/compose/graphics/draw/shapes) and [shared elements](https://developer.android.com/develop/ui/compose/animation/shared-elements).

### Style vocabulary

- **Elastic but controlled:** fast response, subtle overshoot on appropriate properties, minimal lingering oscillation.
- **Morphing, not teleportation:** geometry and context change coherently between states.
- **Spatial continuity:** source elements appear to become destination elements.
- **Expressive feedback:** immediate reactions to press, hover, drag, focus and release.
- **Hierarchy-aware choreography:** nearby/relevant elements move together; delays are short and intentional.
- **Physical interruption:** a gesture can reverse a spring before completion without a new-from-zero animation.

Examples: press-to-scale, pill-to-sheet, card-to-details, active-tab indicator morph, connected-cell regrouping, menus unfolding from their triggering anchors.

## 3. Layered architecture

```text
Application state + Layout destinations + Input
                        |
                        v
               weave-scene (optional)
             identity / shared element / FLIP
                        |
                        v
               weaver-motion (required)
    spring / tween / scheduler / values / tokens
                        |
                        v
        animated presentation snapshot
                        |
            +-----------+-----------+
            |           |           |
         wgpu         Web/DOM     Other adapters

     weave-geometry runs alongside scene/core:
     compatible shapes -> morph values -> paths/meshes
```

### Crates

**weaver-motion:** pure library with `AnimationId`, `Animated<T>`, `Spring`, `Tween`, `MotionToken`, `MotionClock`, `Scheduler`, `AnimationSnapshot`, `AnimationPolicy`. Start with f64 scalars and Vec2; make typed interpolation extensible. Avoid runtime renderer coupling.

**weave-geometry:** normalized geometry types, rounded-rects, rounded polygons, cubic paths, contour correspondence, sampling and validation. CPU-first; tessellation as an adapter concern.

**weave-scene:** stable `ElementId`, source/target bounds, FLIP (First, Last, Invert, Play), clip/z-order/content choreography, layout-to-visual-state bridging, grouping, cancellation and shared-element transitions. Does not own layout or input.

**weave-wgpu:** optional presentation-to-render adapter; shaders, mesh caches, transforms, clipping and invalidation.

**weave-web:** optional adapter, initially using JS/WASM DOM transforms or Canvas; do not require common rendering primitives across platforms.

## 4. Motion primitives and types

`Animatable` should be a sealed or explicitly implemented interpolation/velocity trait. Scalar/vector springs have well-defined velocity. Color and shape paths may use separate interpolation or spring-driven normalized progress.

```rust
pub struct SpringConfig {
    pub mass: f64,
    pub stiffness: f64,
    pub damping_ratio: f64,
    pub position_epsilon: f64,
    pub velocity_epsilon: f64,
}

pub struct ScalarSpring {
    pub value: f64,
    pub velocity: f64,
    pub target: f64,
    pub config: SpringConfig,
}

pub struct PresentationState {
    pub bounds: Rect,
    pub opacity: f32,
    pub scale: Vec2,
    pub rotation_radians: f32,
    pub corner_radii: [f32; 4],
    pub clip: Option<Shape>,
    pub z_index: i32,
}
```

The `PresentationState` above is a conceptual schema. A real implementation also needs layering, render-order and per-property animation metadata. Color blending is performed in a specified color space (prefer linear-light for physically meaningful interpolation, or perceptual spaces where appropriate).

### APIs (proposed)

```rust
let scale = engine.spring("toolbar.create.scale", 1.0, presets.fast_spatial());
scale.set_target(1.06);  // preserves current value and velocity

let panel = engine.animate_bounds("workspace.panel", destination, presets.default_spatial());
let outline = engine.morph("workspace.panel.shape", from, to, presets.expressive_spatial());

engine.shared_transition("document.42", source, destination);
let snapshot = engine.tick(monotonic_timestamp);
renderer.present(&snapshot);
```

Implement APIs incrementally; code above is intentionally not claimed as compiling.

## 5. Physics and time

For each spring value: `m*x'' + c*x' + k*(x-target) = 0`, where `c = 2*zeta*sqrt(k*m)`. Under-damping (`zeta < 1`) overshoots; critical damping (`zeta = 1`) settles fastest without overshoot in the ideal continuous model; over-damping (`zeta > 1`) does not overshoot but settles more slowly.

**Recommended production choice:** exact closed-form damped oscillator update for fixed targets over each elapsed interval. It is less sensitive to frame cadence than numerical integration. For changing targets, solve piecewise from the current position and velocity at each target change. See `references/PHYSICS.md`.

Prototype alternative: semi-implicit Euler with substeps; cap/stabilize large frame gaps rather than simulating unbounded step counts. Do **not** discard physically meaningful elapsed time without choosing and documenting pause/catch-up policy.

Use monotonic time, avoid one timer per widget, and share one animation clock/scheduler. Expose pause/resume, snap-to-end, retarget, reverse, cancellation and settled events. On retarget, maintain current velocity by default; do not restart from progress zero.

### Token design (illustrative, not Google values)

Assuming `mass = 1` and consistent coordinate units:

| Token | Stiffness | Damping ratio | Suggested use |
|---|---:|---:|---|
| `fast_spatial` | 500 | 0.85 | touch/press/focus feedback |
| `default_spatial` | 300 | 0.80 | menus, cards, basic surface transformations |
| `expressive_spatial` | 220 | 0.65 | emphasized transforms with subtle bounce |
| `gentle_spatial` | 140 | 1.00 | larger reading surfaces, restrained motion |
| `effects` | tween | n/a | opacity/color, roughly 150–200ms as starting point |

Tokens are *starting points*; verify perceptually across target distances and device scales. Avoid overshoot on hard layout constraints and clip-sensitive bounds; put restrained emphasis on scale or transform where safe.

## 6. Morphing system

**Level 1: parametric shapes** interpolate rect bounds, per-corner radii, rotation and transforms. Use this first; most button/panel transitions need no polygon solver.

**Level 2: compatible contours** convert contours to compatible cubic Bézier paths or arc-length-resampled point sets; normalize winding and align start points/landmarks; interpolate corresponding points `P_i(t) = (1-t) A_i + t B_i`. Use validated shape correspondence to avoid twisted paths and self-intersection.

**Level 3: topology-aware changes** (later): disconnected contours, holes, merging/splitting shapes. This requires more than naive vertex interpolation and needs fallback transitions if correspondence cannot be established reliably.

Do not confuse **shape morphing** with **shared-element transitions**: moving/scaling a clipping rectangle between screens does not automatically morph its boundary. Both may be composed, but should remain independent primitives.

### Morph pipeline

1. Receive source/destination shape definitions and stable anchor IDs.
2. Resolve common coordinate system and transforms.
3. Check topology, winding, segment compatibility and constraints.
4. Compute correspondence once, cache it by shape pair and version.
5. Advance morph parameter (spring or restrained tween); for arbitrary overshoot, either clamp morph progress to `[0,1]` or define valid extrapolation rules.
6. Interpolate outline and clipping geometry.
7. Tessellate if needed, cache/reuse topology and mesh buffers.
8. Blend/reveal content separately when non-geometric content cannot morph meaningfully.

For background-to-panel conversion, also stage content appearance so text does not stretch along with the surface shape.

## 7. Layout transitions and shared identity

Application layout produces **target bounds**. Weave owns **current presentation bounds**. Do not force full layout recalculation for each purely visual frame if render-only transforms suffice.

Use stable `ElementId` across UI trees. To animate layout reorder, record First bounds, compute Last bounds, render inverted transform, and animate inverse transform to identity (FLIP). Handle parent clipping, z-order, scroll offset, coordinate-space conversion and temporary shared-element overlays.

For a button-to-panel morph: spring position/size, interpolate corners/clip, preserve element identity, crossfade contents while keeping text legible, and keep hit testing aligned with either final layout or animated visuals based on an explicit policy.

Coordinated transitions are timelines/graphs with dependency rules, e.g. start surface expansion immediately, show new text after sufficient interior area exists, fade old icon as text comes in. Avoid excessive stagger and long blocking motion.

## 8. Interaction, accessibility, performance

**Input:** immediate press feedback (< one rendered frame after event when possible), velocity continuity during gesture handoffs, interruption and drag-following behavior. During direct manipulation, driven values can follow pointer input; on release, hand velocity to spring.

**Reduced motion:** respect system preference; snap or use short opacity/color changes, minimize displacement and overshoot, retain focus and navigation cues. Prevent animations from blocking interactions.

**Efficiency:** request frame only while animation is active, use invalidation/dirty regions, prefer transform/opacity operations, cache correspondence and shape tessellation, batch work, and instrument frame time. Aim to meet the platform refresh budget (16.7ms at 60Hz, 8.3ms at 120Hz) including *all* app work, not just physics.

**Lifecycle:** deterministic handling of disappearing/reappearing IDs, device DPI changes, suspend/resume, large time gaps, and renderer teardown. Avoid leaks and stale animation registry entries.

## 9. Deliverable milestones

1. **Core & playground:** analytic spring; vectors; tokens; single clock; retarget and interruption tests.
2. **Simple morphs:** rounded rectangles, pill/panel, clip and content coordination.
3. **Advanced geometry:** compatible path alignment, Bézier interpolation, safety checks, caching.
4. **Scene:** stable IDs, shared-element FLIP, choreography and cancellation semantics.
5. **Adapters:** one end-to-end wgpu prototype, then optional web adapter.
6. **Optimization and compatibility:** profiling, reduced motion, debugging visualization and polished API.

**First demo acceptance test:** a rounded button expands to a panel and reverses on arbitrary click/drag during movement; position and velocity remain continuous; clipping is correct; text remains readable; animation goes idle on settle.

## 10. Deliberate non-goals and risks

Non-goals for v0.1: general-purpose physics simulation, full path topology transformations, full scene graph, automatic semantic inference of arbitrary UI relationships, physics-based rigid-body collisions, or cross-platform exact pixel parity.

Risks: mismatched morph points, GPU tessellation overhead, layout/render desynchronization, clipping artifacts, animation IDs drifting between trees, and overshoot causing invalid geometry. Mitigate through tight MVP scope and explicit fallbacks.

## 11. External references

- [Android Material 3 MotionScheme](https://developer.android.com/reference/kotlin/androidx/compose/material3/MotionScheme)
- [Android Compose shapes](https://developer.android.com/develop/ui/compose/graphics/draw/shapes)
- [Android Compose shared-element transitions](https://developer.android.com/develop/ui/compose/animation/shared-elements)
- [Android Compose animation overview](https://developer.android.com/develop/ui/compose/animation/introduction)

These are conceptual technical references; Android APIs should not be treated as Weave APIs.
