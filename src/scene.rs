use crate::{engine::*, pipeline::PipelineObject, renderer::RenderState};
use winit::{dpi::PhysicalSize, event::WindowEvent};

pub struct Object {}

pub struct Scene {
    pub objects: Vec<Object>,
    pub pipeline_objects: Vec<PipelineObject>,
    pub input_command: Box<dyn Fn(&mut RenderState, &mut PhysicalSize<u32>, &WindowEvent) -> bool>,
    pub init_command: Box<dyn Fn(&mut Engine)>,
    pub render_command: Box<dyn Fn(&Engine) -> Result<(), wgpu::SurfaceError>>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: vec![],
            pipeline_objects: vec![],
            input_command: Box::new(|_, _, _| false),
            init_command: Box::new(|_| {}),
            render_command: Box::new(|_| Ok(())),
        }
    }
}

impl Scene {
    pub fn update_pipelines(&mut self, pipelines: &mut Vec<PipelineObject>) {
        self.pipeline_objects.append(pipelines);
    }
}
