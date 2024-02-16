use crate::pipeline::*;

pub struct RenderState {
    pub clear_color: wgpu::Color,
    pub polygon_fill: bool,
}

pub struct Renderer {
    pub pipeline_manager: PipelineManager,
    pub state: RenderState,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            pipeline_manager: Default::default(),
            state: Default::default(),
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}
