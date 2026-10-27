# Architecture

```text
Application state + Layout destinations + Input
                         |
                         v
                  weave-scene
        identity / shared element / FLIP / choreography
                         |
                         v
                   weaver-motion
        spring / tween / scheduler / values / tokens
                         |
                         v
          animated presentation snapshot
                         |
          +--------------+--------------+
          |              |              |
       wgpu            web/DOM        other adapters

     weave-geometry runs alongside scene/core:
     compatible shapes -> morph values -> paths/meshes
```

## The layers

### `weaver-motion` — the laws of motion

Nothing but numbers and time.

* `spring` — the closed-form solution of the damped oscillator. The result
  depends only on the timestamps, never on the frame rate.
* `value` — `Animated<T>`: a spring driven property of any interpolable
  type.
* `tween` — fixed duration interpolation for effects.
* `tokens` — named presets (`fast_spatial`, `default_spatial`,
  `expressive_spatial`, `gentle_spatial`, `effects`).
* `clock` — the shared monotonic clock and its gap policies.
* `policy` — the accessibility policy (full / reduced / off).
* `scheduler` — the registry of animations; produces `MotionSnapshot`s and
  knows when nothing is moving any more.
* `engine` — the application facing façade.

### `weave-geometry` — shapes

* `rect` / `rounded` — the parametric shapes, with the rules that keep a
  rounded rectangle valid.
* `point` / `path` — points, cubic Bézier segments, contours and paths.
* `correspondence` — resampling by arc length, winding normalisation,
  start-point alignment, self-intersection detection.
* `morph` — `MorphPlan`: the correspondence between two paths, computed
  once and evaluated per frame.
* `cache` — morph plans cached by the versions of the two shapes.

### `weave-scene` — identity and choreography

* `id` — stable element identity. The application provides a key; the
  scene interns it. Identities survive tree rebuilds and are never reused.
* `transform` — FLIP: First, Last, Invert, Play.
* `choreography` — dependencies between the parts of a transition.
* `scene` — the bridge between layout destinations and presentation state.
* `debug` — human readable descriptions of the scene state.

### The adapters

* `weave-wgpu` — a draw list (pure) plus a `wgpu` renderer (optional
  feature).
* `weave-web` — a CSS computation (pure) plus a DOM renderer (optional
  feature).

## The flow of a frame

```text
1. the application changes its state
2. layout produces the destination bounds and shapes
3. the scene associates old and new elements by their stable id
4. the scene produces spring/tween targets (velocity preserved)
5. the scheduler advances every animation from the same timestamp
6. the geometry produces the morphed outlines
7. the adapter draws the snapshot
8. when everything settled, the scene is idle and no frame is requested
```

## Decisions worth knowing

**Exact-time physics over numerical integration.** A closed-form solution
cannot accumulate error, cannot explode after a stall and gives the same
result at 30 Hz and at 144 Hz. The cost is a handful of `exp` calls per
property — 19 ns measured.

**Velocity is never discarded.** `set_target` keeps the current velocity.
The scene therefore never restarts an interrupted animation, and a gesture
can be handed to a spring without a visible jump.

**Destinations and presentation are separate.** Layout decides *where*
things should be; the scene owns *where they are*. A layout pass never has
to run for a purely visual frame.

**Choreography is a dependency graph, not a timeline.** A cue waits for a
condition on the animated state ("the panel is at least half as wide as its
destination"), so the choreography survives interruptions, different frame
rates and different spring presets.

**Morphing is refused when it is unsafe.** A morph that would twist or
self-intersect is not performed; the caller falls back to a crossfade. A
broken outline is worse than a fade.

**Text is never stretched.** The `Bounds` transition mode re-flows the
content, while `Flip` carries the element with a transform. The demo keeps
the label at a fixed font size and reveals it only when the surface is big
enough for it.

## What is deliberately not here

* A window system, a layout engine or a widget toolkit.
* GPU rasterisation and tessellation (adapter concerns).
* Topology changing morphs (holes, splitting, merging): a morph is only
  created between compatible paths.
* Automatic inference of UI relationships: the application declares what
  matters, Weave animates it.
