# Validation and acceptance criteria

## Physics

- Verify analytic solutions for under/critical/over damping against known initial conditions.
- Verify convergence to target and near-zero velocity.
- Change targets mid-flight; position and velocity have no discontinuity at retarget event.
- Compare results at 30, 60, 90, 120 Hz and randomized frame intervals for same timestamps.
- Simulate 1-second suspension and background app policies.
- Check invalid input, NaNs, extreme targets and stiffness/mass limits.
- Ensure settled animations cease scheduling frames.

## Geometry

- Smooth rounded-rect/pill transitions, legal corner radii at every point.
- Winding and landmark alignment before path interpolation.
- Detect invalid self-intersections where relevant or fallback gracefully.
- Closed loops stay closed; bounding boxes and clipping remain finite.
- Test animated progress overshoot and extrapolation policy.

## Scene / UI

- Button -> panel -> button with arbitrary interruption points.
- Fast repeated toggles do not create orphaned animators or ID clashes.
- Reorder list items via stable IDs, including scroll containers and zoomed parents.
- Content reveal/hide remains legible; pointer/focus behavior matches declared policy.
- Reduced-motion setting produces accessible equivalent transitions.

## Performance

- Benchmark hundreds/thousands of simultaneously active scalar springs separately from path morphing.
- Track update time, tessellation time, allocation counts, GPU work and total frame time.
- Validate 60 Hz (16.67ms total frame budget) and 120 Hz (8.33ms total frame budget) under realistic loads; these are targets, not guaranteed results.
- Ensure no redraws in idle state.

## Definition of done for v0.1

A compiled working Rust example must show a button smoothly morphing into a larger panel and back, with reversible spring motion, coordinated shape/clip/content, tests for retarget behavior, reduced-motion option, and a recorded profiling result. A design document or mock API alone does not satisfy completion.
