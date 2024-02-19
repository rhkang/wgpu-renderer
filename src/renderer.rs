use crate::pipeline::*;
use crate::texture::*;

pub struct Renderer {
    pub pipeline_manager: PipelineManager,
    pub state: RenderState,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub texture: Option<Texture>,
    pub bind_group_manager: BindGroupManager,
    pub buffer_manager: BufferManager,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            pipeline_manager: Default::default(),
            state: Default::default(),
            vertex_buffer: None,
            index_buffer: None,
            texture: None,
            bind_group_manager: Default::default(),
            buffer_manager: Default::default(),
        }
    }
}

pub struct RenderState {
    pub clear_color: wgpu::Color,
    pub polygon_fill: bool,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            polygon_fill: true,
            clear_color: wgpu::Color::BLACK,
        }
    }
}

pub struct BindGroupItem {
    pub id: u32,
    pub bind_group: wgpu::BindGroup,
}

pub struct BindGroupManager {
    pub bind_group_list: Vec<BindGroupItem>,
}

impl Default for BindGroupManager {
    fn default() -> Self {
        Self {
            bind_group_list: vec![],
        }
    }
}

impl BindGroupManager {
    pub fn add_bind_group(&mut self, id: u32, bind_group: wgpu::BindGroup) {
        self.bind_group_list.push(BindGroupItem { id, bind_group })
    }

    pub fn find_by_id(&self, id: u32) -> Option<&BindGroupItem> {
        self.bind_group_list.iter().find(|&x| x.id == id)
    }
}

pub struct BufferItem {
    pub id: u32,
    pub buffer: wgpu::Buffer,
}

pub struct BufferManager {
    pub buffer_list: Vec<BufferItem>,
}

impl Default for BufferManager {
    fn default() -> Self {
        Self {
            buffer_list: vec![],
        }
    }
}

impl BufferManager {
    pub fn add_buffer(&mut self, id: u32, buffer: wgpu::Buffer) {
        self.buffer_list.push(BufferItem { id, buffer })
    }

    pub fn find_by_id(&self, id: u32) -> Option<&BufferItem> {
        self.buffer_list.iter().find(|&x| x.id == id)
    }
}
