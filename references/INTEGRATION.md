# API, scene and renderer integration

## Scene IDs

Stable `ElementId` must represent an entity across UI trees. Do not use a transient widget index when lists reorder. Inputs: source element geometry/clip, destination geometry/clip, render constraints and transitions policy.

## Scene flow

1. App changes semantic state.
2. Layout calculates final destination positions and sizes.
3. Weave associates old/new elements by stable ID.
4. Scene produces individual spring/tween targets.
5. Core updates all active values from the same monotonic timestamp.
6. Geometry morphers produce clipping/shape parameters.
7. Renderer draws a consistent snapshot (optionally via a shared-element overlay).
8. When settled, remove temporary overlay/animation records.

FLIP transitions use `First`, `Last`, `Invert`, `Play` transforms. Account for coordinate spaces, scrolling, device scale and parent transforms. Handle disappearance/reappearance as explicit transition types.

## Interaction example

Press creates target `scale=0.96` on pointer down; release target `scale=1.0`; drag may directly drive position. When interrupted, retain current state; transfer current pointer velocity into a released spring if sensible. Do not enforce global bounce on all controls.

## Adapter contract (proposed)

```rust
pub trait MotionRenderer {
    fn present(&mut self, snapshot: &MotionSnapshot);
    fn schedule_frame(&mut self);
}

pub struct MotionSnapshot {
    // Stable ordered presentation records; implementation TBD.
}
```

Prefer implementation exposing immutable snapshots over pushing imperative per-property mutations deep into renderer internals. Propagate invalidations/dirty rectangles to optimize large scenes.

## Accessibility

`MotionPolicy::{Full, Reduced, Off}`. Under reduced motion, prefer near-instant positional snaps and restrained short fades; ensure semantic focus state remains clear. Allow user and platform override. Do not assume replacing motion with opacity is universally safe (flash sensitivity).

## Example choreography

`pill -> panel`: immediately begin bounds+clip morph; fade icon after panel passes an area threshold; reveal text only after enough width exists; on reverse, hide content before constraining the surface. Thresholds should be expressed as scene dependencies, not frame counts.

## Naming

**Weave** (engine), with tentative crates `weave-core`, `weave-geometry`, `weave-scene`, `weave-wgpu`. Exact crates.io names and trademarks are **not confirmed available**. Avoid creating/publicizing package names before checking the registry and legal context.
