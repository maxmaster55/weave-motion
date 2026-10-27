# Profiling

The numbers below were measured on the reference machine with the
dependency free benchmarks that ship with the crates:

```sh
cargo bench -p weaver-motion
cargo bench -p weave-geometry
```

**Machine:** Intel Core i7-10750H @ 2.60 GHz, 12 threads, Linux 6.18,
rustc 1.99.1, `--release` profile (`opt-level = 3`, thin LTO).

## Spring solver

| Measurement | Result |
|---|---|
| single scalar spring | 19.3 ns / update |
| vector spring (2 components) | 31.7 ns / update |
| 100 springs | 0.0026 ms / frame (0.02 % of a 16.7 ms budget) |
| 1 000 springs | 0.0257 ms / frame (0.15 % of a 16.7 ms budget) |
| 10 000 springs | 0.2599 ms / frame (1.56 % of a 16.7 ms budget) |
| scheduler with 100 springs (update **and** snapshot) | 0.0031 ms / frame |
| scheduler with 1 000 springs (update **and** snapshot) | 0.0343 ms / frame |

A spring update is an exact, closed-form evaluation: no iteration, no
allocation, no branching on the frame rate. The cost is therefore linear in
the number of animated properties and independent of how long the frame
took.

For a 120 Hz budget of 8.3 ms, 10 000 springs still use less than 4 % of
the frame.

## Geometry

| Measurement | Result |
|---|---|
| rounded rect interpolation | 0.00001 ms / frame (~10 ns) |
| morph plan (correspondence: resample, align, validate) | 0.124 ms / plan |
| morph evaluation (64 points) | 0.0004 ms / frame (~400 ns) |

The expensive part of a contour morph is the **correspondence**, and it is
computed once per shape pair. That is why `weave-geometry` ships a cache
(`MorphCache`): the plan is built when the transition starts and evaluated
per frame afterwards. A cached plan costs 0.4 µs per frame, which is
negligible next to the rasterisation of the shape itself.

## Idle behaviour

The scheduler never asks for a frame while every animation is settled. The
idle check is a linear scan over the registered animations, so the cost of
an idle frame is proportional to the number of *elements*, not to the
number of frames that were skipped.

## What is not measured here

* **Rasterisation and tessellation.** Those are adapter concerns. The
  `weave-wgpu` adapter draws one instanced quad per surface and evaluates
  the rounded rectangle in the fragment shader, so the CPU never
  tessellates a shape.
* **A full application frame.** The budgets above cover the motion engine
  only. An application has to fit its own layout, painting and event
  handling into the rest of the frame.
* **Different machines.** The absolute numbers depend on the CPU; the
  *shape* of the numbers does not: the solver is O(1) per property and
  the correspondence is O(n²) in the number of contour samples.
