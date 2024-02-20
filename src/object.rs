use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub struct Object {
    pub mesh: Mesh,
    pub position: [f32; 3],
}

pub type IndexType = u16;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<IndexType>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Mesh {
    pub fn create_mesh(device: &wgpu::Device, vertices: Vec<Vertex>, indices: Vec<IndexType>) -> Self{
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub fn get_mesh_from_model(device: &wgpu::Device, file_path: &str, scale: f32) -> Mesh {
    let model_scene = russimp::scene::Scene::from_file(file_path, vec![]).unwrap();

    let mesh = model_scene.meshes.first().unwrap();

    let _vertices: Vec<[f32;3]> = mesh.vertices.clone().iter().map(|v| { [v.x * scale, v.y * scale, v.z * scale] }).collect();

    let mut _tex_coords = vec![];
    for tex in mesh.texture_coords.clone() {
        match tex {
            Some(v) => {
                for vec in v {
                    _tex_coords.push([vec.x, 1.0 - vec.y]);
                }
            }
            None => {},
        }
    }

    let mut _indices = vec![];
    for face_iter_item in mesh.faces.iter().clone() {
        for face in &face_iter_item.0 {
            _indices.push(face.clone());
        }
    }

    assert!(_vertices.len() == _tex_coords.len());

    let mut vertices = vec![];
    for i in 0.._vertices.len() {
        vertices.push(Vertex{ position: _vertices[i], color: [f32::default(); 3], tex_coords: _tex_coords[i] });
    }

    let mut indices = vec![];
    for i in 0.._indices.len() {
        indices.push(_indices[i] as IndexType);
    }

    Mesh::create_mesh(device, vertices, indices)
}

pub fn get_object_from_mesh(mesh: Mesh, position: [f32; 3]) -> Object {
    Object{
        mesh,
        position,
    }
}
