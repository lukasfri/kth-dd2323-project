use std::f32::consts;
use std::mem;

use crate::{camera::Camera, scene::Scene};

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use super::renderer::Renderer;

pub struct Rasterizer {
    pub props: WgpuRenderProps,
}

impl Rasterizer {
    pub fn new(props: WgpuRenderProps) -> Self {
        Self { props }
    }
}

impl Renderer for Rasterizer {
    type Error = anyhow::Error;
    type Canvas<'window> = RenderSurface<'window>;

    fn render(
        &mut self,
        canvas: &mut Self::Canvas<'_>,
        scene: &Scene,
        camera: &Camera,
    ) -> Result<(), Self::Error> {
        canvas.draw(&mut self.props)?;

        let vertex_data = scene
            .triangles
            .iter()
            .flat_map(|triangle| {
                vec![
                    Vertex::new(triangle.v0.into(), triangle.color.into()),
                    Vertex::new(triangle.v1.into(), triangle.color.into()),
                    Vertex::new(triangle.v2.into(), triangle.color.into()),
                ]
            })
            .collect::<Vec<Vertex>>();

        let mut index_data = (0..vertex_data.len() as u16).collect::<Vec<u16>>();
        index_data.extend((0..vertex_data.len() as u16).map(|i| vertex_data.len() as u16 - i - 1));

        let vertex_buf = canvas
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buf = canvas
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.props.mesh_props = Some(MeshProps {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
        });

        self.props.aspect_ratio =
            canvas.surface_config.width as f32 / canvas.surface_config.height as f32;

        let mx_total = WgpuRenderProps::generate_matrix(
            self.props.aspect_ratio,
            Into::<[f32; 3]>::into(camera.position).into(),
            Into::<[f32; 3]>::into(camera.direction_vector()).into(),
        );

        let mx_ref: &[f32; 16] = mx_total.as_ref();

        canvas
            .queue
            .write_buffer(&self.props.uniform_buf, 0, bytemuck::cast_slice(mx_ref));

        Ok(())
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            // 4.
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum ShaderBinding {
    PerspectiveTransform = 0,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _color: Color,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl From<crate::Color> for Color {
    fn from(color: crate::Color) -> Self {
        Self {
            r: color.0.x,
            g: color.0.y,
            b: color.0.z,
            a: 1.0,
        }
    }
}

impl Color {
    pub const fn from_rgba(data: [f32; 4]) -> Self {
        Self {
            r: data[0],
            g: data[1],
            b: data[2],
            a: data[3],
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(data: [f32; 4]) -> Self {
        Self::from_rgba(data)
    }
}

impl Vertex {
    fn new(pos: [f32; 3], color: Color) -> Self {
        Self {
            _pos: [pos[0], pos[1], pos[2], 1.0],
            _color: color,
        }
    }

    const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub struct MeshProps {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: usize,
}

pub struct WgpuRenderProps {
    pub mesh_props: Option<MeshProps>,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_wire: Option<wgpu::RenderPipeline>,
    pub depth_texture: Texture,
    pub aspect_ratio: f32,
}

impl WgpuRenderProps {
    pub fn generate_matrix(aspect_ratio: f32, position: Vec3, direction: Vec3) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_infinite_rh(consts::FRAC_PI_4, aspect_ratio, 1.0);

        let direction = (direction).normalize();

        let focal_point = position + direction;

        let view = glam::Mat4::look_at_rh(position, focal_point, glam::Vec3::Z);

        projection * view
    }

    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
    ) -> Self {
        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: ShaderBinding::PerspectiveTransform as u32,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let aspect_ratio = config.width as f32 / config.height as f32;

        // Create other resources
        let mx_total = Self::generate_matrix(
            aspect_ratio,
            glam::Vec3::new(1.5f32, -5.0, 3.0),
            glam::Vec3::X,
        );
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: ShaderBinding::PerspectiveTransform as u32,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        const SHADER_SOURCE: &str = include_str!("./shader.wgsl");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        let vertex_buffers = [Vertex::desc()];

        let depth_texture = Texture::create_depth_texture(device, config, "depth_texture");

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(config.view_formats[0].into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_wire = if device
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_wire",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.view_formats[0],
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            Some(pipeline_wire)
        } else {
            None
        };

        // Done
        WgpuRenderProps {
            aspect_ratio,
            mesh_props: None,
            bind_group,
            uniform_buf,
            pipeline,
            pipeline_wire,
            depth_texture,
        }
    }

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            if let Some(MeshProps {
                vertex_buf,
                index_buf,
                index_count,
            }) = self.mesh_props.as_ref()
            {
                rpass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, vertex_buf.slice(..));
                rpass.pop_debug_group();
                rpass.insert_debug_marker("Draw!");
                rpass.draw_indexed(0..(*index_count) as u32, 0, 0..1);
                if let Some(ref pipe) = self.pipeline_wire {
                    rpass.set_pipeline(pipe);
                    rpass.draw_indexed(0..(*index_count) as u32, 0, 0..1);
                }
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}

pub struct RenderSurface<'window> {
    pub surface: wgpu::Surface<'window>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl RenderSurface<'_> {
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn draw(&mut self, props: &mut WgpuRenderProps) -> anyhow::Result<()> {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        props.render(&view, &self.device, &self.queue);

        frame.present();

        Ok(())
    }
}
