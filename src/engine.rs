use std::sync::Arc;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

use crate::renderer::*;
use crate::scene::*;

impl Default for RenderState {
    fn default() -> Self {
        Self {
            polygon_fill: true,
            clear_color: wgpu::Color::BLACK,
        }
    }
}

pub struct Engine {
    // wgpu graphics components
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    // window, surface
    window: Arc<Window>,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub size: winit::dpi::PhysicalSize<u32>,

    // managing objects
    pub renderer: Renderer,
    pub scene: Scene,
}

impl Engine {
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
        (&self.scene.input_command)(&mut self.renderer.state, &mut self.size, event)
    }

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

        Self {
            surface,
            device,
            window,
            config,
            queue,
            size,
            renderer: Default::default(),
            scene: Default::default(),
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        (self.scene.render_command)(self)
    }

    pub fn update(&self) {}
}

pub async fn run(scene: Option<Scene>) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let mut engine = Engine::new(window).await;
    match &scene {
        Some(s) => {
            let f = &s.init_command;
            (f)(&mut engine);

            let scene_data = scene.unwrap();
            engine.scene.render_command = scene_data.render_command;
            engine.scene.input_command = scene_data.input_command;
        }
        None => {
            eprintln!("No Entry Scene provided");
            return;
        }
    };

    // move pipelines provided by scene into renderer
    loop {
        match engine.scene.pipeline_objects.pop() {
            Some(item) => engine.renderer.pipeline_manager.add(item),
            None => break,
        }
    }

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
                        engine.renderer.state.polygon_fill = !engine.renderer.state.polygon_fill;
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
