# Weave — expressive motion engine

**Weave** is a portable, renderer-independent Rust motion engine. It brings
spring-based, interruption-safe transitions, shape morphing and shared-element
choreography to applications including **Weaver** and **Loom**.

> Weave is inspired by the *concepts* behind Material 3 Expressive
> (spring-driven spatial motion, separate effects motion, shape morphing,
> shared-element transitions). It is not a Google product, port or
> endorsement, and its presets are **not** official Material tokens.

## Why Weave exists

Motion that restarts from zero on every interruption feels broken. Weave is
built around a small number of hard rules:

* **Retargetable** — a new target never snaps a moving value or discards its
  velocity.
* **Exact-time physics** — springs use a closed-form damped-oscillator
  solution, so the result does not depend on frame cadence.
* **Idle at rest** — no frames are requested while every animation is settled.
* **Portable** — the core has no third-party dependencies, builds on
  `no_std` + `alloc`, and the adapters (GPU, web) are strictly optional.

## Workspace layout

```text
weave/
├── crates/
│   ├── weave-core/      # springs, tweens, tokens, clock, scheduler (no deps)
│   ├── weave-geometry/  # normalized geometry, rounded-rect + Bézier morphing
│   ├── weave-scene/     # stable element identity, FLIP, choreography
│   ├── weave-wgpu/      # optional GPU adapter
│   └── weave-web/       # optional WASM/DOM adapter
├── examples/
│   ├── demo-support/    # shared terminal helpers for the demos
│   ├── spring-playground/   # interactive spring playground (terminal)
│   └── button-to-panel/     # pill -> panel morph demo (terminal)
├── docs/                # architecture, portability and profiling guides
├── flake.nix            # `nix develop` / `nix build` / `nix flake check`
└── Makefile             # portable task runner (cargo only)
```

## Quick start

```bash
nix develop          # recommended on NixOS
# or, with a plain toolchain:
cargo build --workspace
cargo test --workspace
cargo run -p spring-playground
cargo run -p button-to-panel
```

Without Nix:

```sh
make build      # cargo build --workspace
make test       # cargo test --workspace
make check      # fmt + clippy + tests
```

## Using Weave from another project

The crates are plain Cargo packages with **path + version** metadata, so they
can be consumed in three ways:

```toml
# 1. from a local checkout
weave-core = { path = "../weave_engine/crates/weave-core" }

# 2. from git (no publish required)
weave-core = { git = "https://github.com/weave-motion/weave", package = "weave-core" }

# 3. from crates.io, once published
weave-core = "0.1"
```

A five line example:

```rust
use weave_core::{Animated, SpringConfig, Vec2};

let config = SpringConfig::unit_mass(300.0, 0.8)?;
let mut scale = Animated::new(Vec2::ONE, Vec2::splat(1.06), config);

scale.advance(1.0 / 60.0);
let velocity = scale.velocity();

// Interrupt: the destination changes, the motion continues.
scale.set_target(Vec2::ONE);
assert_eq!(scale.velocity(), velocity);
# Ok::<(), weave_core::SpringError>(())
```

See [`docs/PORTABILITY.md`](docs/PORTABILITY.md) for the full guide: `no_std`
usage, feature flags, adapter isolation and the Nix workflow.

## Documentation

| Document | Contents |
|---|---|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | the layers, the frame flow, the decisions |
| [`docs/PORTABILITY.md`](docs/PORTABILITY.md) | consuming Weave, features, `no_std`, Nix |
| [`docs/PROFILING.md`](docs/PROFILING.md) | measured numbers and their budgets |
| [`DESIGN.md`](DESIGN.md) | the original product and engineering specification |
| [`SKILL.md`](SKILL.md) | the agent skill for implementing and using Weave |
| [`references/`](references) | physics, morphing, integration and testing notes |
| [`CHANGELOG.md`](CHANGELOG.md) | what changed |

## Status

| Milestone | Status |
|---|---|
| 1. Core & playground: analytic spring, tokens, clock, retarget tests | ✅ |
| 2. Simple morphs: rounded rects, pill/panel, clip + content coordination | ✅ |
| 3. Advanced geometry: contour alignment, Bézier interpolation, caching | ✅ |
| 4. Scene: stable IDs, shared-element FLIP, choreography, cancellation | ✅ |
| 5. Adapters: `weave-wgpu` prototype, `weave-web` (wasm) adapter | ✅ |
| 6. Optimization & compatibility: profiling, reduced motion, polished API | ✅ |

### Known limits

* The `wgpu` adapter compiles and is structured like a real renderer, but it
  could not be executed in the reference environment (no GPU, no display
  server). The `weave-web` adapter was compiled for `wasm32-unknown-unknown`,
  not executed in a browser.
* Topology changing morphs (holes, splitting, merging) are out of scope: a
  morph is only created between compatible paths, otherwise the caller
  crossfades.
* The presets are starting points. They still need perceptual tuning per
  product, per distance and per device scale.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
