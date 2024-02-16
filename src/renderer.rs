use crate::pipeline::*;
use crate::texture::*;

pub struct RenderState {
    pub clear_color: wgpu::Color,
    pub polygon_fill: bool,
}

pub struct Renderer {
    pub pipeline_manager: PipelineManager,
    pub state: RenderState,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub texture: Option<Texture>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub uniform_bind_group: Option<wgpu::BindGroup>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            pipeline_manager: Default::default(),
            state: Default::default(),
            vertex_buffer: None,
            index_buffer: None,
            texture: None,
            bind_group: None,
            uniform_bind_group: None,
        }
    }
}
