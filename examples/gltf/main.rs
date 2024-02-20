use bytemuck::{Pod, Zeroable};
use cgmath::SquareMatrix;
use std::default::Default;
use wgpu::util::DeviceExt;
use wgpu_renderer::camera::CameraUniform;
use wgpu_renderer::engine::*;
use wgpu_renderer::pipeline::PipelineObject;
use wgpu_renderer::scene::*;
use wgpu_renderer::{object, pipeline, texture};
use winit::event::WindowEvent;

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

pub fn input(engine: &mut Engine, event: &WindowEvent) -> bool {
    (&mut engine.scene.camera_controller).process_window_events(event)
}

pub fn init(engine: &mut Engine) {
    let scene = &mut engine.scene;
    let device = &engine.device;
    let config = &engine.config;
    let queue = &engine.queue;

    let fox_object = object::get_object_from_mesh(object::get_mesh_from_model(device, "resources/fox/Fox.gltf", 0.02), [0.0; 3]);
    scene.objects.push(fox_object);

    let diffuse_bytes = include_bytes!("../../resources/fox/Texture.png");
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
        source: wgpu::ShaderSource::Wgsl(include_str!("gltf.wgsl").into()),
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
            buffers: &[object::Vertex::desc()],
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
        depth_stencil: pipeline::create_depth_stencil_state(),
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
            buffers: &[object::Vertex::desc()],
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
        depth_stencil: pipeline::create_depth_stencil_state(),
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
        contents: bytemuck::cast_slice(&scene.objects.first().unwrap().mesh.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&scene.objects.first().unwrap().mesh.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    engine.renderer.vertex_buffer = Some(vertex_buffer);
    engine.renderer.index_buffer = Some(index_buffer);

    let depth = texture::Texture::create_depth_texture(device, config, None);
    engine.renderer.depth_texture = Some(depth);
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

    let depth_stencil_attachment = match &engine.renderer.depth_texture {
        None => None,
        Some(texture) => Some(wgpu::RenderPassDepthStencilAttachment{
            view: &texture.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    };

    let fox_object = engine.scene.objects.first().unwrap();

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
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if engine.renderer.state.polygon_fill {
            _render_pass.set_pipeline(&engine.renderer.pipeline_manager.first_polygon_fill_item());
        } else {
            _render_pass.set_pipeline(&engine.renderer.pipeline_manager.first_polygon_line_item());
        }

        _render_pass.set_vertex_buffer(0, fox_object.mesh.vertex_buffer.slice(..));
        _render_pass.set_index_buffer(fox_object.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

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

        _render_pass.draw_indexed(0..fox_object.mesh.indices.len() as u32, 0, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}

fn update(engine: &mut Engine) {
    let renderer = &mut engine.renderer;
    let scene = &mut engine.scene;
    let queue = &engine.queue;

    scene.camera_controller.update_camera(&mut scene.camera);
    scene
        .camera_uniform
        .update_transformation(&mut scene.camera);

    queue.write_buffer(
        &renderer.buffer_manager.find_by_id(0).unwrap().buffer,
        0,
        bytemuck::cast_slice(&[scene.camera_uniform]),
    );

    let elapsed = engine.start_time.elapsed().unwrap().as_secs_f32();
    queue.write_buffer(
        &renderer.buffer_manager.find_by_id(1).unwrap().buffer,
        0,
        bytemuck::cast_slice(&[RotationUniform {
            rotation: cgmath::Matrix4::from_angle_y(cgmath::Rad(elapsed)).into(),
        }]),
    );
}

fn main() {
    let scene = Scene {
        camera: Default::default(),
        camera_controller: Default::default(),
        camera_uniform: Default::default(),
        objects: vec![],
        pipeline_objects: vec![],
    };

    let commands = CommandBundle {
        input_command: Box::new(input),
        init_command: Box::new(init),
        render_command: Box::new(render),
        update_command: Box::new(update),
    };

    pollster::block_on(run(Some(scene), commands));
}
