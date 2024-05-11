use std::f32::consts;
use std::mem;

use crate::{camera::Camera, renderer::Renderer, scene::Scene};

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

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

        self.props.vertex_buf = vertex_buf;
        self.props.index_buf = index_buf;
        self.props.index_count = index_data.len();

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
        // self.props.uniform_buf.slice(..)(canvas.queue, 0, bytemuck::cast_slice(mx_ref));

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
    Texture = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    // _tex_coord: [f32; 2],
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

impl From<super::Color> for Color {
    fn from(color: super::Color) -> Self {
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
    fn new(
        pos: [f32; 3],
        // tc: [i8; 2],
        color: Color,
        // asd
    ) -> Self {
        Self {
            _pos: [pos[0], pos[1], pos[2], 1.0],
            // _tex_coord: [tc[0] as f32, tc[1] as f32],
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

fn vertex(pos: [i8; 3], color: Color) -> Vertex {
    Vertex::new(pos.map(|a| a as f32), color)
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    pub const RED: Color = Color::from_rgba([1.0, 0.0, 0.0, 1.0]);
    pub const PRIMARY: Color = RED;
    // pub const GREEN: Color = Color::from_rgba([0.0, 1.0, 0.0, 1.0]);
    // pub const BLACK: Color = Color::from_rgba([0.0, 0.0, 1.0, 1.0]);
    pub const BLUE: Color = Color::from_rgba([0.0, 0.0, 1.0, 0.0]);
    pub const THIRD: Color = BLUE;
    // pub const YELLOW: Color = Color::from_rgba([1.0, 1.0, 1.0, 1.0]);
    pub const SECOND: Color = THIRD;
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], THIRD),
        vertex([1, -1, 1], PRIMARY),
        vertex([1, 1, 1], SECOND),
        vertex([-1, 1, 1], PRIMARY),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], PRIMARY),
        vertex([1, 1, -1], PRIMARY),
        vertex([1, -1, -1], PRIMARY),
        vertex([-1, -1, -1], PRIMARY),
        // right (1, 0, 0)
        vertex([1, -1, -1], PRIMARY),
        vertex([1, 1, -1], PRIMARY),
        vertex([1, 1, 1], SECOND),
        vertex([1, -1, 1], PRIMARY),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], THIRD),
        vertex([-1, 1, 1], PRIMARY),
        vertex([-1, 1, -1], PRIMARY),
        vertex([-1, -1, -1], PRIMARY),
        // front (0, 1, 0)
        vertex([1, 1, -1], PRIMARY),
        vertex([-1, 1, -1], PRIMARY),
        vertex([-1, 1, 1], PRIMARY),
        vertex([1, 1, 1], SECOND),
        // back (0, -1, 0)
        vertex([1, -1, 1], PRIMARY),
        vertex([-1, -1, 1], THIRD),
        vertex([-1, -1, -1], PRIMARY),
        vertex([1, -1, -1], PRIMARY),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size)
        .map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}

pub struct WgpuRenderProps {
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: usize,
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
        queue: &wgpu::Queue,
    ) -> Self {
        // Create the vertex and index buffers
        let (mut vertex_data, mut index_data) = create_vertices();
        let (mut vertex_data_2, mut index_data_2) = create_vertices();

        // Shift positions of the second cube to the right
        // Shift indexes of the second cube
        index_data_2
            .iter_mut()
            .for_each(|i| *i += vertex_data.len() as u16);

        vertex_data_2.iter_mut().for_each(|v| v._pos[1] += 3.0);

        vertex_data.extend(vertex_data_2);
        index_data.extend(index_data_2);

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: ShaderBinding::PerspectiveTransform as u32,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: ShaderBinding::Texture as u32,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create the texture
        let size = 256u32;
        let texels = create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size),
                rows_per_image: None,
            },
            texture_extent,
        );

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
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        const SHADER_SOURCE: &str = include_str!("../examples/shader.wgsl");

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
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
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
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
            if let Some(ref pipe) = self.pipeline_wire {
                rpass.set_pipeline(pipe);
                rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
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
