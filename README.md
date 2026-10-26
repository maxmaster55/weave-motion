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
* **Portable** — the core has no third-party dependencies and builds on
  `no_std` + `alloc`; adapters (GPU, web) are strictly optional.

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
│   ├── spring-playground/   # interactive spring playground (terminal)
│   └── button-to-panel/     # pill -> panel morph demo (terminal)
├── docs/                # architecture + portability guides
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

See [`docs/PORTABILITY.md`](docs/PORTABILITY.md) for the full guide: `no_std`
usage, feature flags, adapter isolation and the Nix workflow.

## Specification documents

* [`DESIGN.md`](DESIGN.md): complete product and engineering design specification.
* [`SKILL.md`](SKILL.md): agent skill entry point for implementing and using Weave.
* [`references/PHYSICS.md`](references/PHYSICS.md): spring mathematics, numerical strategies, timing, presets.
* [`references/MORPHING.md`](references/MORPHING.md): geometry and shape-matching algorithms.
* [`references/INTEGRATION.md`](references/INTEGRATION.md): API, scene transitions, adapters, accessibility.
* [`references/TESTING.md`](references/TESTING.md): test matrix and completion criteria.
* [`IMPLEMENTATION_PROMPT.md`](IMPLEMENTATION_PROMPT.md): ready-to-paste instruction for a coding agent.

## Status

| Milestone | Status |
|---|---|
| 1. Core & playground: analytic spring, tokens, clock, retarget tests | 🚧 in progress |
| 2. Simple morphs: rounded rects, pill/panel, clip + content coordination | ⬜ planned |
| 3. Advanced geometry: contour alignment, Bézier interpolation, caching | ⬜ planned |
| 4. Scene: stable IDs, shared-element FLIP, choreography, cancellation | ⬜ planned |
| 5. Adapters: `weave-wgpu` prototype, `weave-web` (wasm) adapter | ⬜ planned |
| 6. Optimization & compatibility: profiling, reduced motion, polished API | ⬜ planned |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
