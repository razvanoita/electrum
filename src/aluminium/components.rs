use cgmath::*;

use crate::pewter;

pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>,
    pub scale: cgmath::Vector3<f32>,
}

pub struct Mesh {
    pub vertex_buffer: pewter::VertexBuffer,
    pub index_buffer: pewter::IndexBuffer,
}

pub enum Component {
    TransformComponent(Transform),
    MeshComponent(Mesh),
}

pub enum ComponentMask {
    TransformComponent = 0b0000_0000_0000_0001,
    MeshComponent = 0b0000_0000_0000_0010
}

