use crate::camera::{Camera, CameraController, CameraUniform, Projection};
use crate::object::*;
use crate::pipeline::PipelineObject;

pub struct Scene {
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub projection: Projection,
    pub objects: Vec<Object>,
    pub pipeline_objects: Vec<PipelineObject>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            camera_controller: CameraController::new(5.0, 1.0),
            camera_uniform: CameraUniform::new(),
            projection: Projection::new(1, 1, cgmath::Deg(45.0), 0.1, 100.0),
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
