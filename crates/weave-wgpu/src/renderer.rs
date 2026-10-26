//! The `wgpu` backend.
//!
//! One instanced pipeline draws every surface: a quad per item, with the
//! rounded rectangle evaluated in the fragment shader. The shape is never
//! tessellated on the CPU, which keeps the per-frame cost independent of
//! the number of corners.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::draw_list::DrawList;
use weave_scene::MotionRenderer;

/// The WGSL source of the surface shader.
pub const SHADER: &str = r#"
struct Uniforms {
    viewport: vec2<f32>,
};

struct Instance {
    bounds: vec4<f32>,
    radii: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read> instances: array<Instance>;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radii: vec4<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex: u32,
    @builtin(instance_index) instance: u32,
) -> VertexOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );

    let corner = corners[vertex];
    let data = instances[instance];
    let half = data.bounds.zw * 0.5;
    let center = data.bounds.xy + half;
    let offset = (corner * 2.0 - vec2<f32>(1.0, 1.0)) * half;

    var out: VertexOut;
    out.position = vec4<f32>(
        center.x / uniforms.viewport.x * 2.0 - 1.0,
        1.0 - center.y / uniforms.viewport.y * 2.0,
        0.0,
        1.0,
    );
    out.local = offset;
    out.half_size = half;
    out.radii = data.radii;
    out.color = data.color;
    return out;
}

fn rounded_box(p: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    // radii: top-left, top-right, bottom-right, bottom-left
    var radius = radii.x;
    if (p.x > 0.0 && p.y < 0.0) {
        radius = radii.y;
    } else if (p.x > 0.0 && p.y >= 0.0) {
        radius = radii.z;
    } else if (p.x <= 0.0 && p.y >= 0.0) {
        radius = radii.w;
    }

    let q = abs(p) - half_size + vec2<f32>(radius, radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - radius;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let distance = rounded_box(in.local, in.half_size, in.radii);
    let coverage = clamp(0.5 - distance, 0.0, 1.0);
    return vec4<f32>(in.color.rgb, in.color.a * coverage);
}
"#;

/// The data of one drawn surface.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Instance {
    bounds: [f32; 4],
    radii: [f32; 4],
    color: [f32; 4],
}

/// The data that is constant for a frame.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Uniforms {
    viewport: [f32; 2],
}

/// The errors the renderer can fail with.
#[derive(Debug)]
pub enum RendererError {
    /// No adapter matched the request.
    NoAdapter,
    /// The adapter refused to create a device.
    Device(String),
    /// The surface configuration was rejected.
    Surface(String),
}

impl core::fmt::Display for RendererError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RendererError::NoAdapter => f.write_str("no graphics adapter was found"),
            RendererError::Device(message) => write!(f, "could not create a device: {message}"),
            RendererError::Surface(message) => {
                write!(f, "could not configure the surface: {message}")
            }
        }
    }
}

impl std::error::Error for RendererError {}

/// A `wgpu` renderer for the Weave presentation state.
pub struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    capacity: u64,
    draw_list: Option<DrawList>,
}

impl WgpuRenderer {
    /// Creates a renderer for a viewport of the given size.
    pub async fn new(
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY | wgpu::Backends::GL,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RendererError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("weave device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .map_err(|error| RendererError::Device(error.to_string()))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("weave surface shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("weave bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("weave pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("weave surface pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("weave uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let capacity = 64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("weave instances"),
            size: capacity * std::mem::size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("weave bind group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instance_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            layout,
            uniform_buffer,
            instance_buffer,
            bind_group,
            format,
            width,
            height,
            capacity,
            draw_list: None,
        })
    }

    /// The width of the viewport, in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of the viewport, in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The texture format the renderer draws into.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Resizes the viewport.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
    }

    /// Draws the draw list into `target`.
    pub fn render(&mut self, list: &DrawList, target: &wgpu::TextureView) {
        if list.items.is_empty() {
            return;
        }

        self.upload(list);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("weave frame"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("weave pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..6, 0..list.items.len() as u32);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    /// Draws the draw list into a texture of its own.
    ///
    /// The texture is returned so that it can be copied to the CPU, which
    /// is how the headless tests verify the output.
    pub fn render_to_texture(&mut self, list: &DrawList) -> wgpu::Texture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("weave offscreen target"),
            size: wgpu::Extent3d {
                width: self.width.max(1),
                height: self.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.render(list, &view);

        texture
    }

    /// The device the renderer draws with.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// The draw list that was last presented.
    pub fn draw_list(&self) -> Option<&DrawList> {
        self.draw_list.as_ref()
    }

    /// Renders the last presented draw list into `target`.
    pub fn render_presented(&mut self, target: &wgpu::TextureView) {
        if let Some(list) = self.draw_list.clone() {
            self.render(&list, target);
        }
    }

    fn upload(&mut self, list: &DrawList) {
        let uniforms = Uniforms {
            viewport: [list.width.max(1.0) as f32, list.height.max(1.0) as f32],
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let instances: Vec<Instance> = list
            .items
            .iter()
            .map(|item| Instance {
                bounds: [
                    item.bounds.x as f32,
                    item.bounds.y as f32,
                    item.bounds.width as f32,
                    item.bounds.height as f32,
                ],
                radii: [
                    item.radii.top_left as f32,
                    item.radii.top_right as f32,
                    item.radii.bottom_right as f32,
                    item.radii.bottom_left as f32,
                ],
                color: [item.color.r, item.color.g, item.color.b, item.alpha()],
            })
            .collect();

        let needed = instances.len() as u64;
        if needed > self.capacity {
            self.capacity = needed.next_power_of_two();
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("weave instances"),
                size: self.capacity * std::mem::size_of::<Instance>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("weave bind group"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.instance_buffer.as_entire_binding(),
                    },
                ],
            });
        }

        if !instances.is_empty() {
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        }
    }
}

impl MotionRenderer for WgpuRenderer {
    fn present(&mut self, snapshot: &weave_scene::SceneSnapshot) {
        self.draw_list = Some(DrawList::from_snapshot(snapshot));
    }

    fn schedule_frame(&mut self) {
        // Frame scheduling is the responsibility of the host: the engine
        // reports `needs_frame`, the host calls `render`.
    }
}

/// A convenience constructor for a buffer initialised from data.
#[allow(dead_code)]
fn buffer_from_data(device: &wgpu::Device, label: &str, data: &[u8]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: data,
        usage: wgpu::BufferUsages::COPY_DST,
    })
}
