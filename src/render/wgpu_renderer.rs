use crate::backend::surface_provider::SurfaceProvider;
use crate::backend::window::WindowConfig;
use crate::core::assets::ImageId;
use crate::math::vec2::Vec2;
use crate::render::SpriteDrawData;
use crate::render::Vertex as CoreVertex;
use crate::render::renderer::{RenderError, RenderResult, Renderer};
use raw_window_handle::{DisplayHandle, WindowHandle};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct WgpuRenderer {
    size: (u32, u32),
    instance: Option<wgpu::Instance>,
    surface: Option<wgpu::Surface<'static>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    clear_color: wgpu::Color,
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    pending_vertices: Vec<VertexGPU>,
    sprite_pipeline: Option<wgpu::RenderPipeline>,
    sprite_vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    sprite_bind_group_layout: Option<wgpu::BindGroupLayout>,
    textures: HashMap<ImageId, TextureGpu>,
    sprite_draws: Vec<SpriteDraw>,
}

impl WgpuRenderer {
    pub fn new() -> Self {
        WgpuRenderer {
            size: (0, 0),
            instance: None,
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            config: None,
            clear_color: wgpu::Color::WHITE,
            pipeline: None,
            vertex_buffer_layout: VertexGPU::buffer_layout(),
            pending_vertices: Vec::new(),
            sprite_pipeline: None,
            sprite_vertex_buffer_layout: SpriteVertexGPU::buffer_layout(),
            sprite_bind_group_layout: None,
            textures: HashMap::new(),
            sprite_draws: Vec::new(),
        }
    }

    fn device(&self) -> &wgpu::Device {
        self.device.as_ref().expect("wgpu device not initialized")
    }
    fn queue(&self) -> &wgpu::Queue {
        self.queue.as_ref().expect("wgpu queue not initialized")
    }
    fn surface(&self) -> &wgpu::Surface<'static> {
        self.surface.as_ref().expect("wgpu surface not initialized")
    }
    fn config(&self) -> &wgpu::SurfaceConfiguration {
        self.config.as_ref().expect("wgpu config not initialized")
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct VertexGPU {
    pos: [f32; 2],
    color: [f32; 4],
}

impl VertexGPU {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexGPU>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 8,
                    shader_location: 1,
                },
            ],
        }
    }
}

struct TextureGpu {
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteVertexGPU {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl SpriteVertexGPU {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertexGPU>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 8,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 2,
                },
            ],
        }
    }
}

struct SpriteDraw {
    texture_id: ImageId,
    vertices: [SpriteVertexGPU; 6],
}

impl Renderer for WgpuRenderer {
    fn init(
        &mut self,
        surface_provider: &dyn SurfaceProvider,
        config: Option<&WindowConfig>,
    ) -> RenderResult<()> {
        self.size = surface_provider.size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let wh: WindowHandle = surface_provider
            .window_handle()
            .map_err(|e| RenderError::InitFailed(format!("window_handle failed: {}", e)))?;
        let dh: DisplayHandle = surface_provider
            .display_handle()
            .map_err(|e| RenderError::InitFailed(format!("display_handle failed: {}", e)))?;
        let unsafe_target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_window_handle: wh.as_raw(),
            raw_display_handle: dh.as_raw(),
        };
        let surface = unsafe { instance.create_surface_unsafe(unsafe_target) }
            .map_err(|e| RenderError::SurfaceError(format!("create_surface failed: {}", e)))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .map_err(|e| RenderError::InitFailed(format!("no compatible adapter found: {}", e)))?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("RustyEngine Device"),
            trace: wgpu::Trace::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
        }))
        .map_err(|e| RenderError::InitFailed(format!("request_device failed: {}", e)))?;

        let caps = surface.get_capabilities(&adapter);
        let vsync_enabled = config.and_then(|cfg| cfg.vsync).unwrap_or(false);
        let present_mode = if vsync_enabled {
            [
                wgpu::PresentMode::Fifo,
                wgpu::PresentMode::FifoRelaxed,
                wgpu::PresentMode::AutoVsync,
                wgpu::PresentMode::Mailbox,
            ]
            .into_iter()
            .find(|mode| caps.present_modes.iter().any(|m| m == mode))
            .unwrap_or(caps.present_modes[0])
        } else {
            [
                wgpu::PresentMode::AutoNoVsync,
                wgpu::PresentMode::Immediate,
                wgpu::PresentMode::Mailbox,
            ]
            .into_iter()
            .find(|mode| caps.present_modes.iter().any(|m| m == mode))
            .unwrap_or_else(|| {
                [
                    wgpu::PresentMode::Fifo,
                    wgpu::PresentMode::FifoRelaxed,
                    wgpu::PresentMode::AutoVsync,
                ]
                .into_iter()
                .find(|mode| caps.present_modes.iter().any(|m| m == mode))
                .unwrap_or(caps.present_modes[0])
            })
        };
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let alpha_mode = caps.alpha_modes[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: self.size.0.max(1),
            height: self.size.1.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 0,
        };
        surface.configure(&device, &config);

        // Inline WGSL shader to render pre-transformed, colored vertices
        let shader_src = r#"
            struct VsOut {
                @builtin(position) pos: vec4<f32>,
                @location(0) color: vec4<f32>,
            };

            @vertex
            fn vs(@location(0) pos: vec2<f32>, @location(1) color: vec4<f32>) -> VsOut {
                var out: VsOut;
                out.pos = vec4<f32>(pos, 0.0, 1.0);
                out.color = color;
                return out;
            }

            @fragment
            fn fs(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
                return color;
            }
        "#;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("immediate shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("immediate pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("immediate pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: std::slice::from_ref(&self.vertex_buffer_layout),
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            cache: None,
            multiview: None,
        });

        // Sprite pipeline (textured quads)
        let sprite_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let sprite_shader_src = r#"
            struct SpriteVsIn {
                @location(0) pos: vec2<f32>,
                @location(1) uv: vec2<f32>,
                @location(2) color: vec4<f32>,
            };

            struct SpriteVsOut {
                @builtin(position) pos: vec4<f32>,
                @location(0) uv: vec2<f32>,
                @location(1) color: vec4<f32>,
            };

            @group(0) @binding(0) var sprite_tex: texture_2d<f32>;
            @group(0) @binding(1) var sprite_sampler: sampler;

            @vertex
            fn vs_main(input: SpriteVsIn) -> SpriteVsOut {
                var out: SpriteVsOut;
                out.pos = vec4<f32>(input.pos, 0.0, 1.0);
                out.uv = input.uv;
                out.color = input.color;
                return out;
            }

            @fragment
            fn fs_main(input: SpriteVsOut) -> @location(0) vec4<f32> {
                let tex_color = textureSample(sprite_tex, sprite_sampler, input.uv);
                return tex_color * input.color;
            }
        "#;
        let sprite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(sprite_shader_src.into()),
        });

        let sprite_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sprite pipeline layout"),
                bind_group_layouts: &[&sprite_bind_group_layout],
                push_constant_ranges: &[],
            });

        let sprite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&sprite_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sprite_shader,
                entry_point: Some("vs_main"),
                buffers: std::slice::from_ref(&self.sprite_vertex_buffer_layout),
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sprite_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            cache: None,
            multiview: None,
        });

        self.instance = Some(instance);
        self.surface = Some(surface);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.pipeline = Some(pipeline);
        self.sprite_bind_group_layout = Some(sprite_bind_group_layout);
        self.sprite_pipeline = Some(sprite_pipeline);

        Ok(())
    }

    fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        if let (Some(surface), Some(device), Some(config)) = (
            self.surface.as_ref(),
            self.device.as_ref(),
            self.config.as_mut(),
        ) {
            config.width = self.size.0.max(1);
            config.height = self.size.1.max(1);
            surface.configure(device, config);
        }
    }

    fn present(&mut self) -> RenderResult<()> {
        let surface = self.surface();
        let device = self.device();
        let queue = self.queue();
        let config = self.config();

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
                surface.configure(device, config);
                surface.get_current_texture().map_err(|e| {
                    RenderError::SurfaceError(format!("get_current_texture after Lost: {}", e))
                })?
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("GPU memory exhausted in present");
                return Err(RenderError::OutOfMemory);
            }
            Err(_) => return Ok(()),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("clear encoder"),
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("main pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if !self.pending_vertices.is_empty() {
            let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("immediate vb"),
                contents: bytemuck::cast_slice(&self.pending_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            rpass.set_pipeline(self.pipeline.as_ref().unwrap());
            rpass.set_vertex_buffer(0, vb.slice(..));
            rpass.draw(0..(self.pending_vertices.len() as u32), 0..1);
        }

        if !self.sprite_draws.is_empty() {
            let sprite_pipeline = self.sprite_pipeline.as_ref().unwrap();
            let bind_group_layout = self.sprite_bind_group_layout.as_ref().unwrap();
            rpass.set_pipeline(sprite_pipeline);

            for draw in &self.sprite_draws {
                if let Some(texture) = self.textures.get(&draw.texture_id) {
                    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("sprite bind group"),
                        layout: bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&texture.view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&texture.sampler),
                            },
                        ],
                    });

                    let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("sprite vb"),
                        contents: bytemuck::cast_slice(&draw.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_vertex_buffer(0, vb.slice(..));
                    rpass.draw(0..6, 0..1);
                }
            }
        }
        drop(rpass);

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        self.pending_vertices.clear();
        self.sprite_draws.clear();
        Ok(())
    }
    fn submit(&mut self, vertices: &[CoreVertex]) {
        for v in vertices.iter().copied() {
            self.pending_vertices.push(VertexGPU {
                pos: v.pos,
                color: v.color,
            });
        }
    }
    fn set_clear_color(&mut self, rgba: [f32; 4]) {
        self.clear_color = wgpu::Color {
            r: rgba[0] as f64,
            g: rgba[1] as f64,
            b: rgba[2] as f64,
            a: rgba[3] as f64,
        };
    }

    fn upload_image(
        &mut self,
        id: ImageId,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> RenderResult<()> {
        let device = self.device();
        let queue = self.queue();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sprite texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.textures.insert(id, TextureGpu { view, sampler });

        Ok(())
    }

    fn draw_sprites(&mut self, sprites: &[SpriteDrawData], viewport_size: (u32, u32)) {
        let (w, h) = (viewport_size.0.max(1) as f32, viewport_size.1.max(1) as f32);

        for sprite in sprites {
            if !self.textures.contains_key(&sprite.image_id) {
                continue;
            }

            // Calculate world corners from sprite data
            let corners = self.compute_sprite_corners(sprite);

            let to_ndc = |p: Vec2| -> [f32; 2] { [(p.x / w) * 2.0 - 1.0, 1.0 - (p.y / h) * 2.0] };

            let tl = to_ndc(corners[0]);
            let tr = to_ndc(corners[1]);
            let br = to_ndc(corners[2]);
            let bl = to_ndc(corners[3]);

            let color: [f32; 4] = sprite.tint.to_linear_rgba();

            let uv_min = [sprite.uv_min.x, sprite.uv_min.y];
            let uv_max = [sprite.uv_max.x, sprite.uv_max.y];

            let vertices = [
                SpriteVertexGPU {
                    pos: tl,
                    uv: [uv_min[0], uv_min[1]],
                    color,
                },
                SpriteVertexGPU {
                    pos: tr,
                    uv: [uv_max[0], uv_min[1]],
                    color,
                },
                SpriteVertexGPU {
                    pos: br,
                    uv: [uv_max[0], uv_max[1]],
                    color,
                },
                SpriteVertexGPU {
                    pos: tl,
                    uv: [uv_min[0], uv_min[1]],
                    color,
                },
                SpriteVertexGPU {
                    pos: br,
                    uv: [uv_max[0], uv_max[1]],
                    color,
                },
                SpriteVertexGPU {
                    pos: bl,
                    uv: [uv_min[0], uv_max[1]],
                    color,
                },
            ];

            self.sprite_draws.push(SpriteDraw {
                texture_id: sprite.image_id,
                vertices,
            });
        }
    }
}

impl WgpuRenderer {
    /// Compute world-space corners of a sprite quad from draw data.
    fn compute_sprite_corners(&self, sprite: &SpriteDrawData) -> [Vec2; 4] {
        let size = sprite.size;
        let origin_px = Vec2::new(sprite.origin.x * size.x, sprite.origin.y * size.y);

        // Local corners (unscaled, unrotated)
        let local_tl = Vec2::new(0.0, 0.0) - origin_px;
        let local_tr = Vec2::new(size.x, 0.0) - origin_px;
        let local_br = Vec2::new(size.x, size.y) - origin_px;
        let local_bl = Vec2::new(0.0, size.y) - origin_px;

        // Apply scale, rotation, and translation
        let cos_r = sprite.rotation.cos();
        let sin_r = sprite.rotation.sin();

        let transform = |p: Vec2| -> Vec2 {
            let scaled = Vec2::new(p.x * sprite.scale.x, p.y * sprite.scale.y);
            let rotated = Vec2::new(
                scaled.x * cos_r - scaled.y * sin_r,
                scaled.x * sin_r + scaled.y * cos_r,
            );
            rotated + sprite.position
        };

        [
            transform(local_tl),
            transform(local_tr),
            transform(local_br),
            transform(local_bl),
        ]
    }
}
