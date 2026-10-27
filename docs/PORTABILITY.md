# Portability

Weave is meant to be used by *other* projects: Weaver, Loom and whatever
comes next. This document describes what "portable" means here and how to
consume the engine from another project.

## The crates

| Crate | Dependencies | Purpose |
|---|---|---|
| `weaver-motion` | none (`libm` only for `no_std`) | springs, tweens, tokens, clock, policy, scheduler, engine |
| `weave-geometry` | `weaver-motion` | rectangles, rounded shapes, Bézier paths, morphing, caching |
| `weave-scene` | `weaver-motion`, `weave-geometry` | stable identity, FLIP, choreography |
| `weave-wgpu` | `weave-*` (+ `wgpu` behind a feature) | GPU adapter |
| `weave-web` | `weave-*` (+ `wasm-bindgen` behind a feature) | DOM adapter |

The three core crates have **no third-party dependencies** in their default
build. The adapters keep their heavy dependencies behind a feature flag,
so a project that only needs the physics never downloads a GPU stack.

## Consuming Weave

Every crate carries both a `path` and a `version`, so all of these work:

```toml
# 1. from a checkout next to your project
weaver-motion = { path = "../weaver-motion/crates/weaver-motion" }

# 2. from git, without publishing
weaver-motion = { git = "https://github.com/weave-motion/weave", package = "weaver-motion" }

# 3. from crates.io, once published
weaver-motion = "0.1"
```

Only `weaver-motion` is needed for the physics. `weave-geometry` and
`weave-scene` add shapes and layout transitions. The adapters are
optional.

### Feature flags

```toml
# the default: std, everything enabled
weaver-motion = "0.1"

# no_std: for embedded targets, custom engines or wasm without std shims
weaver-motion = { version = "0.1", default-features = false, features = ["libm"] }
```

| Feature | Crates | Meaning |
|---|---|---|
| `std` | core, geometry, scene | use the standard library (default) |
| `libm` | core | pure-Rust math for `no_std` builds |
| `wgpu` | weave-wgpu | the GPU backend |
| `web` | weave-web | the DOM bindings |

`no_std` builds need the `libm` feature, because `exp` and `sqrt` are not
part of `core`. The build fails with a clear message if the feature is
missing.

## Portability rules the code follows

1. **No wall clock.** The engine only ever sees monotonic timestamps that
   the host provides. `weaver_motion::MotionClock` decides what a large gap
   means (fast-forward, freeze or snap).
2. **No rendering dependency in the core.** The core knows about numbers,
   not pixels: it never imports `wgpu`, `web-sys` or a UI framework.
3. **No `unsafe`.** Every crate sets `#![deny(unsafe_code)]`.
4. **No allocation in the hot path.** The spring solver is closed form and
   allocation free; the scheduler reuses its buffers.
5. **No absolute paths, no environment variables, no network.** The
   examples are pure computation and the demos need only a terminal.
6. **Deterministic results.** The same timestamps produce the same values
   on every platform, which is what makes a motion engine testable.

## The Nix workflow

The repository ships a flake for NixOS users:

```sh
nix develop          # a shell with cargo, rustc, rustfmt, clippy, rust-analyzer
nix build            # the portable demos
nix flake check      # build + test the whole workspace in the sandbox
nix run .#spring-playground
nix run .#button-to-panel
```

The dev shell also provides `wasm-pack` and `rustup`, so the wasm adapter
can be built with:

```sh
rustup target add wasm32-unknown-unknown
cargo build -p weave-web --features web --target wasm32-unknown-unknown
```

`shell.nix` is the same environment for people who prefer `nix-shell`.

## Without Nix

Everything works with a plain Rust toolchain:

```sh
make build      # cargo build --workspace
make test       # cargo test --workspace
make check      # fmt + clippy + tests
```

The CI workflow runs the same checks on Linux, macOS and Windows with the
stable toolchain and with the MSRV (1.75), which is what keeps the
portability promise honest.

## Adding a new target platform

The engine itself is platform independent. Only the adapters need platform
work:

| Adapter | What it needs |
|---|---|
| `weave-wgpu` | a GPU and a `wgpu` backend (Vulkan, Metal, DX12, GL) |
| `weave-web` | a browser (`wasm32-unknown-unknown`) |

If your platform needs something else, implement
`weaver_motion::MotionRenderer` and turn a scene snapshot into whatever your
renderer draws. The `weave-wgpu` draw list (`weave_wgpu::DrawList`) is a
good template: it is pure data and needs no GPU.
