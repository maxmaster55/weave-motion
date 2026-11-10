//! # weave-wgpu
//!
//! A GPU rendering adapter for the Weave motion engine.
//!
//! The crate has two layers:
//!
//! * `draw_list` turns a [`SceneSnapshot`](weave_scene::SceneSnapshot)
//!   into a list of draw items. That part is pure data manipulation: it
//!   needs no GPU, so it is testable on a build machine.
//! * the `wgpu` feature adds a renderer that draws those items with
//!   `wgpu`. The GPU backend is **optional**: the crate builds without it
//!   and without any third-party dependency.
//!
//! ```no_run
//! # #[cfg(feature = "wgpu")] {
//! use weave_wgpu::WgpuRenderer;
//!
//! # async fn example() -> Result<(), weave_wgpu::RendererError> {
//! let renderer = WgpuRenderer::new(800, 600, wgpu::TextureFormat::Rgba8UnormSrgb).await?;
//! # let _ = renderer;
//! # Ok(())
//! # }
//! # }
//! ```

#![warn(missing_docs)]

mod draw_list;

pub use draw_list::{Color, DrawItem, DrawList};

#[cfg(feature = "wgpu")]
mod renderer;

#[cfg(feature = "wgpu")]
pub use renderer::{RendererError, WgpuRenderer};

/// Version of the Weave engine this crate belongs to.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
