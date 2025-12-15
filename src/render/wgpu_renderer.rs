use crate::core::surface_provider::SurfaceProvider;
use crate::core::vertex::Vertex as CoreVertex;
use crate::render::renderer::{RenderError, RenderResult, Renderer};
use raw_window_handle::{DisplayHandle, WindowHandle};
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

impl Renderer for WgpuRenderer {
    fn init(&mut self, surface_provider: &dyn SurfaceProvider) -> RenderResult<()> {
        self.size = surface_provider.size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let wh: WindowHandle = surface_provider.window_handle().map_err(|_| RenderError)?;
        let dh: DisplayHandle = surface_provider.display_handle().map_err(|_| RenderError)?;
        let unsafe_target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_window_handle: wh.as_raw(),
            raw_display_handle: dh.as_raw(),
        };
        let surface =
            unsafe { instance.create_surface_unsafe(unsafe_target) }.map_err(|_| RenderError)?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .map_err(|_| RenderError)?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("RustyEngine Device"),
            trace: wgpu::Trace::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
        }))
        .map_err(|_| RenderError)?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let present_mode = caps
            .present_modes
            .iter()
            .copied()
            .find(|m| *m == wgpu::PresentMode::Fifo)
            .unwrap_or(wgpu::PresentMode::AutoVsync);
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

        self.instance = Some(instance);
        self.surface = Some(surface);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.pipeline = Some(pipeline);

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
                surface.get_current_texture().map_err(|_| RenderError)?
            }
            Err(wgpu::SurfaceError::OutOfMemory) => return Err(RenderError),
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
        drop(rpass);

        queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        self.pending_vertices.clear();
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
}
