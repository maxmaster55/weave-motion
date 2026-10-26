# Changelog

All notable changes to this project are documented here. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — unreleased

The first release of the Weave motion engine: a portable, renderer
independent motion system for UI applications.

### Added

#### Core (`weave-core`)

* Exact-time damped spring solver: the closed-form solution of
  `m x'' + c x' + k(x - target) = 0`, with underdamped, critically damped
  and overdamped branches.
* `Animated<T>`: a spring driven property of any interpolable type
  (`f64`, `Vec2`, `Vec4` and anything an application adds).
* `Tween` / `TweenConfig` / `Easing`, including CSS-style cubic bezier
  curves, for effects such as opacity and colour.
* `MotionToken` presets: `fast_spatial`, `default_spatial`,
  `expressive_spatial`, `gentle_spatial` and `effects`.
* `MotionClock` with explicit gap policies (fast-forward, freeze, snap) for
  suspensions and debugger pauses.
* `MotionPolicy` (full / reduced / off) for accessible motion.
* `Scheduler`: a registry of animations that produces immutable
  `MotionSnapshot`s and reports when nothing is moving any more.
* `MotionEngine`: the application facing façade.
* `MotionRenderer`: the contract every adapter implements.
* `no_std` support (with the `libm` feature) and zero third-party
  dependencies in the default build.

#### Geometry (`weave-geometry`)

* `Rect`, `Radii` and `RoundedRect` with CSS-like validity rules: a
  radius never exceeds half of the smaller side and the two radii of an
  edge never add up to more than the edge.
* `Shape` and `Morph`: parametric morphing with a clamped progress, so a
  spring overshoot can never produce an invalid shape.
* `Point`, `CubicSegment`, `Contour` and `Path`.
* `MorphPlan`: contour morphing with arc-length resampling, winding
  normalisation, start-point alignment and self-intersection detection. A
  plan is refused when the result would be unsafe.
* `MorphCache`: morph plans cached by the versions of the two shapes.

#### Scene (`weave-scene`)

* `ElementRegistry` and `ElementId`: stable identity that survives tree
  rebuilds; identifiers are never reused.
* `Flip`, `Transform` and `TransitionMode`: the First-Last-Invert-Play
  technique, in both `Bounds` and `Flip` flavours.
* `Scene`: destinations in, presentation out; interruption keeps the
  velocity.
* `Choreographer`, `Gate` and `Cue`: dependencies instead of delays.
* `Scene::debug_report()`: human readable state for debugging.

#### Adapters

* `weave-wgpu`: a renderer independent draw list plus an optional `wgpu`
  backend that evaluates rounded rectangles in the fragment shader.
* `weave-web`: a CSS computation plus an optional DOM renderer.

#### Examples

* `spring-playground`: a motion report and an interactive playground.
* `button-to-panel`: the pill-to-panel demo, with clipping, content
  choreography, interruption, a velocity handoff and reduced motion.

#### Tooling

* A Nix flake: `nix develop`, `nix build`, `nix flake check`, `nix run`.
* A `Makefile` and POSIX shell scripts for toolchain-only environments.
* A CI matrix covering Linux, macOS and Windows with the stable toolchain
  and the MSRV.
* Dependency free benchmarks for the spring solver and the geometry layer.

[0.1.0]: https://github.com/weave-motion/weave/releases/tag/v0.1.0
