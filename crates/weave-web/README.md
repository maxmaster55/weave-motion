# weave-web

Web (wasm/DOM) rendering adapter for the [Weave](../../README.md) motion engine.

## How it works

Weave computes the motion; the adapter only *applies* it. The DOM adapter
never uses the Web Animations API: the browser is told where every element
is, and the engine decides how it gets there. Interruption, reduced motion
and frame pacing stay under the control of the engine.

## Layers

| Layer | Needs a browser | Contents |
|---|---|---|
| `weave_web::DomStyle` | no | the CSS of a presentation state |
| `weave_web::DomRenderer` | yes | applies the styles to the DOM |

The style computation is pure string building, so it is covered by tests
that run on any target.

## Features

* `web` — enables the DOM bindings (`wasm-bindgen`, `web-sys`). Without it
  the crate has **no third-party dependencies at all**.

```toml
[dependencies]
weave-web = { version = "0.1", features = ["web"] }
```

## Building for the browser

```sh
rustup target add wasm32-unknown-unknown
cargo build -p weave-web --features web --target wasm32-unknown-unknown
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
