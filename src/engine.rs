use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::window::{Window, WindowBuilder};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::pipeline::*;
use crate::scene::*;

pub struct RenderState{
    pub clear_color: wgpu::Color,
    pub polygon_fill: bool,
}

impl Default for RenderState{
    fn default() -> Self {
        Self { polygon_fill: true, clear_color: wgpu::Color::BLACK }
    }
}

pub struct Engine{
    // wgpu graphics components
    device: wgpu::Device,
    queue: wgpu::Queue,
    
    // window, surface
    window: Arc<Window>,
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    size: winit::dpi::PhysicalSize<u32>,
    
    // managing objects
    pub pipeline_manager: PipelineManager,
    pub render_state: RenderState,
    scene: Scene,
}

impl Engine{
    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.render_state.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };

                true
            }
            _ => false,
        }
    }

    // pub fn graphics_wgpu_objects(&self) -> (&wgpu::Device, &wgpu::Surface, &wgpu::Queue) {
    //     (&self.device, &self.surface, &self.queue)
    // }

    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::POLYGON_MODE_LINE,
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 3,
        };
        surface.configure(&device, &config);

        let pipeline_manager = PipelineManager { pipelines: vec![] };

        Self {
            surface,
            device,
            window,
            config,
            queue,
            pipeline_manager,
            size,
            render_state: Default::default(),
            scene: Default::default(),
        }
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        (&self.scene.render_command)(&self.device, &self.surface, &self.queue, &self)
    }

    pub fn update(&self) {
    }    
}

pub async fn run(scene: Option<Scene>) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let mut engine = Engine::new(window).await;
    match &scene {
        Some(s) => {
            let f = &s.init_command;
            (f)(&mut engine.scene, &engine.device, &engine.config);

            engine.scene.render_command = scene.unwrap().render_command;
        },
        None => {},
    };

    engine.pipeline_manager.add(engine.scene.pipeline_objects.pop().unwrap());
    engine.pipeline_manager.add(engine.scene.pipeline_objects.pop().unwrap());

    let _ = event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == engine.window().id() => {
            if !engine.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        control_flow.exit();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Space),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        engine.render_state.polygon_fill = !engine.render_state.polygon_fill;
                    }
                    WindowEvent::Resized(new_size) => engine.resize(*new_size),
                    WindowEvent::RedrawRequested => {
                        engine.update();
                        match engine.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => engine.resize(engine.size),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::AboutToWait => {
            engine.window().request_redraw();
        }
        _ => (),
    });
}
