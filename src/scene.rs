use crate::{engine, pipeline::PipelineObject};

pub struct Object {

}

pub struct Scene {
    pub objects: Vec<Object>,
    pub pipeline_objects: Vec<PipelineObject>,
    pub init_command: Box<dyn Fn(&mut Scene, &wgpu::Device, &wgpu::SurfaceConfiguration)>,
    pub render_command: Box<dyn Fn(&wgpu::Device, &wgpu::Surface, &wgpu::Queue, &engine::Engine) -> Result<(), wgpu::SurfaceError>>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: vec![],
            pipeline_objects: vec![],
            init_command: Box::new(|_, _, _| {}),
            render_command: Box::new(|_, _, _, _| { Ok(()) }),
        }
    }
}

impl Scene{
    pub fn update_pipelines(&mut self, pipelines: &mut Vec<PipelineObject>) {
        self.pipeline_objects.append(pipelines);
    }
}