use wgpu::util::DeviceExt;
use wgpu_renderer::engine::*;
use wgpu_renderer::pipeline::PipelineObject;
use wgpu_renderer::scene::*;
use winit::event::WindowEvent;

use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

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
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.1, 0.2, 0.4],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.2, 0.3, 0.4],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.6, 0.0, 0.4],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.7, 0.8, 0.4],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.9, 0.4],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub fn input(engine: &mut Engine, event: &WindowEvent) -> bool {
    let renderer = &mut engine.renderer;
    let size = &mut engine.size;

    const FACTOR: f64 = 0.3;
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            renderer.state.clear_color = wgpu::Color {
                r: position.x as f64 / size.width as f64 + FACTOR,
                g: position.y as f64 / size.height as f64 + FACTOR,
                b: 1.0,
                a: 1.0,
            };

            true
        }
        _ => false,
    }
}

pub fn init(engine: &mut Engine) {
    let scene = &mut engine.scene;
    let device = &engine.device;
    let config = &engine.config;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("triangles.wgsl").into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[],
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

        // _render_pass.draw(0..VERTICES.len() as u32, 0..1);   // If vertex only
        _render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
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
        update_command: Box::new(|_| {}),
    };

    pollster::block_on(run(Some(scene), commands));
}
