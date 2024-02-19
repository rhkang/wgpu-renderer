use crate::camera::{Camera, CameraController, CameraUniform};
use crate::object::*;
use crate::pipeline::PipelineObject;

pub struct Scene {
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub objects: Vec<Object>,
    pub pipeline_objects: Vec<PipelineObject>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            camera_controller: Default::default(),
            camera_uniform: CameraUniform::new(),
            objects: vec![],
            pipeline_objects: vec![],
        }
    }
}

impl Scene {
    pub fn update_pipelines(&mut self, pipelines: &mut Vec<PipelineObject>) {
        self.pipeline_objects.append(pipelines);
    }
}
