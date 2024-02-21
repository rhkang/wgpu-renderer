use bytemuck::{Pod, Zeroable};
use cgmath::SquareMatrix;
use std::default::Default;
use wgpu::util::DeviceExt;
use wgpu_renderer::camera;
use wgpu_renderer::camera::CameraUniform;
use wgpu_renderer::engine::*;
use wgpu_renderer::pipeline::PipelineObject;
use wgpu_renderer::scene::*;
use wgpu_renderer::texture;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct RotationUniform {
    rotation: [[f32; 4]; 4],
}

impl RotationUniform {
    fn new() -> Self {
        Self {
            rotation: cgmath::Matrix4::identity().into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    }, // A
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    }, // B
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    }, // C
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    }, // D
];

const INDICES: &[u16] = &[0, 2, 3, 0, 3, 1];

pub fn init(engine: &mut Engine) {
    let scene = &mut engine.scene;
    let device = &engine.device;
    let config = &engine.config;
    let queue = &engine.queue;

    let diffuse_bytes = include_bytes!("../../resources/texture_1.jpg");
    let texture = texture::Texture::from_bytes(device, queue, diffuse_bytes, None).unwrap();

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
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

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
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

    let camera = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        contents: bytemuck::cast_slice(&[CameraUniform::new()]),
    });

    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(camera.as_entire_buffer_binding()),
        }],
    });

    let rotation_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[RotationUniform::new()]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let rotation_uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let rotation_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &rotation_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(
                rotation_uniform_buffer.as_entire_buffer_binding(),
            ),
        }],
    });

    engine
        .renderer
        .bind_group_manager
        .add_bind_group(0, bind_group);
    engine
        .renderer
        .bind_group_manager
        .add_bind_group(1, camera_bind_group);
    engine
        .renderer
        .bind_group_manager
        .add_bind_group(2, rotation_uniform_bind_group);

    engine.renderer.buffer_manager.add_buffer(0, camera);

    engine
        .renderer
        .buffer_manager
        .add_buffer(1, rotation_uniform_buffer);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("uniform.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[
            &bind_group_layout,
            &camera_bind_group_layout,
            &rotation_uniform_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let pipeline_line = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Line,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let mut pipelines = vec![
        PipelineObject {
            pipeline,
            polygon_mode: wgpu::PolygonMode::Fill,
        },
        PipelineObject {
            pipeline: pipeline_line,
            polygon_mode: wgpu::PolygonMode::Line,
        },
    ];
    scene.update_pipelines(&mut pipelines);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    engine.renderer.vertex_buffer = Some(vertex_buffer);
    engine.renderer.index_buffer = Some(index_buffer);
}

pub fn render(engine: &Engine) -> Result<(), wgpu::SurfaceError> {
    let surface = &engine.surface;
    let device = &engine.device;
    let queue = &engine.queue;

    let output = surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(engine.renderer.state.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if engine.renderer.state.polygon_fill {
            _render_pass.set_pipeline(&engine.renderer.pipeline_manager.first_polygon_fill_item());
        } else {
            _render_pass.set_pipeline(&engine.renderer.pipeline_manager.first_polygon_line_item());
        }

        match &engine.renderer.vertex_buffer {
            Some(v) => _render_pass.set_vertex_buffer(0, v.slice(..)),
            None => {}
        }

        match &engine.renderer.index_buffer {
            Some(i) => _render_pass.set_index_buffer(i.slice(..), wgpu::IndexFormat::Uint16),
            None => {}
        }

        _render_pass.set_bind_group(
            0,
            &engine
                .renderer
                .bind_group_manager
                .find_by_id(0)
                .unwrap()
                .bind_group,
            &[],
        );
        _render_pass.set_bind_group(
            1,
            &engine
                .renderer
                .bind_group_manager
                .find_by_id(1)
                .unwrap()
                .bind_group,
            &[],
        );
        _render_pass.set_bind_group(
            2,
            &engine
                .renderer
                .bind_group_manager
                .find_by_id(2)
                .unwrap()
                .bind_group,
            &[],
        );

        _render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}

fn update(engine: &mut Engine) {
    let renderer = &mut engine.renderer;
    let scene = &mut engine.scene;
    let queue = &engine.queue;

    let dt = engine.last_render_time.elapsed();
    engine.last_render_time = instant::Instant::now();

    scene.camera_controller.update_camera(&mut scene.camera, dt);
    scene
        .camera_uniform
        .update_view_proj(&mut scene.camera, &scene.projection);

    queue.write_buffer(
        &renderer.buffer_manager.find_by_id(0).unwrap().buffer,
        0,
        bytemuck::cast_slice(&[scene.camera_uniform]),
    );

    let elapsed = engine.start_time.elapsed().as_secs_f32();
    queue.write_buffer(
        &renderer.buffer_manager.find_by_id(1).unwrap().buffer,
        0,
        bytemuck::cast_slice(&[RotationUniform {
            rotation: cgmath::Matrix4::from_angle_z(cgmath::Rad(elapsed)).into(),
        }]),
    );
}

fn main() {
    let mut scene = Scene::default();
    scene.camera = camera::Camera::new(
        [0.0, 0.0, 2.0],
        cgmath::Rad(-camera::SAFE_FRAC_PI_2),
        cgmath::Rad(0.0),
    );

    let commands = CommandBundle {
        input_command: Box::new(camera::input),
        init_command: Box::new(init),
        render_command: Box::new(render),
        update_command: Box::new(update),
    };

    pollster::block_on(run(Some(scene), commands));
}
