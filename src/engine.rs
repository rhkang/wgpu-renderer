use crate::renderer::*;
use crate::scene::*;

use std::sync::Arc;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

pub struct CommandBundle {
    pub input_command: Box<dyn Fn(&mut Engine, &WindowEvent) -> bool>,
    pub init_command: Box<dyn Fn(&mut Engine)>,
    pub render_command: Box<dyn Fn(&Engine) -> Result<(), wgpu::SurfaceError>>,
    pub update_command: Box<dyn Fn(&mut Engine)>,
}

impl CommandBundle {
    pub fn set_bundle(&mut self, commands: CommandBundle) {
        self.set_input_command(commands.input_command);
        self.set_init_command(commands.init_command);
        self.set_render_command(commands.render_command);
        self.set_update_command(commands.update_command);
    }

    pub fn set_input_command(&mut self, command: Box<dyn Fn(&mut Engine, &WindowEvent) -> bool>) {
        self.input_command = command;
    }

    pub fn set_init_command(&mut self, command: Box<dyn Fn(&mut Engine)>) {
        self.init_command = command;
    }

    pub fn set_render_command(
        &mut self,
        command: Box<dyn Fn(&Engine) -> Result<(), wgpu::SurfaceError>>,
    ) {
        self.render_command = command;
    }

    pub fn set_update_command(&mut self, command: Box<dyn Fn(&mut Engine)>) {
        self.update_command = command;
    }
}

impl Default for CommandBundle {
    fn default() -> Self {
        Self {
            input_command: Box::new(|_, _| false),
            init_command: Box::new(|_| {}),
            render_command: Box::new(|_| Ok(())),
            update_command: Box::new(|_| {}),
        }
    }
}

pub struct Game {
    pub engine: Engine,

    // commands
    pub commands: CommandBundle,
}

impl Game {
    fn input(&mut self, event: &WindowEvent) -> bool {
        (&self.commands.input_command)(&mut self.engine, event)
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        (&self.commands.render_command)(&mut self.engine)
    }

    pub fn update(&mut self) {
        (&self.commands.update_command)(&mut self.engine)
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
    pub start_time: std::time::SystemTime,
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
            device,
            queue,
            window,
            surface,
            config,
            size,
            renderer: Default::default(),
            scene: Default::default(),
            start_time: std::time::SystemTime::now(),
        }
    }
}

pub async fn run(scene: Option<Scene>, commands: CommandBundle) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let engine = Engine::new(window).await;
    let mut game = Game { engine, commands };

    game.engine
        .scene
        .camera
        .set_aspect(game.engine.size.width as f32 / game.engine.size.height as f32);

    match &scene {
        Some(_) => {
            (&game.commands.init_command)(&mut game.engine);
        }
        None => {
            eprintln!("No Entry Scene provided");
            return;
        }
    };

    // move pipelines provided by scene into renderer
    loop {
        match game.engine.scene.pipeline_objects.pop() {
            Some(item) => game.engine.renderer.pipeline_manager.add(item),
            None => break,
        }
    }

    let _ = event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == game.engine.window().id() => {
            if !game.input(event) {
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
                        game.engine.renderer.state.polygon_fill =
                            !game.engine.renderer.state.polygon_fill;
                    }
                    WindowEvent::Resized(new_size) => {
                        game.engine.resize(*new_size);
                        game.engine
                            .scene
                            .camera
                            .set_aspect(new_size.width as f32 / new_size.height as f32);
                    }
                    WindowEvent::RedrawRequested => {
                        game.update();
                        match game.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => game.engine.resize(game.engine.size),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::AboutToWait => {
            game.engine.window().request_redraw();
        }
        _ => (),
    });
}
