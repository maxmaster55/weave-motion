# weave-wgpu

GPU rendering adapter for the [Weave](../../README.md) motion engine.

## Layers

| Layer | Needs a GPU | Contents |
|---|---|---|
| `weave_wgpu::DrawList` | no | turns a scene snapshot into draw items |
| `weave_wgpu::WgpuRenderer` | yes | draws those items with `wgpu` |

The draw list is pure data manipulation. That keeps it testable on a build
machine and lets an application inspect what *would* be drawn before it
reaches the GPU.

## Features

* `wgpu` — enables the GPU backend. Without it the crate has **no
  third-party dependencies at all**.

```toml
[dependencies]
weave-wgpu = { version = "0.1", features = ["wgpu"] }
```

## Usage

```rust,no_run
use weave_wgpu::WgpuRenderer;

# async fn example() -> Result<(), weave_wgpu::RendererError> {
let mut renderer = WgpuRenderer::new(800, 600, wgpu::TextureFormat::Rgba8UnormSrgb).await?;
# let _ = &mut renderer;
# Ok(())
# }
```

The renderer draws one instanced quad per surface and evaluates the rounded
rectangle in the fragment shader, so the CPU never tessellates a shape.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
