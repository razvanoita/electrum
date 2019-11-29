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

#[derive(Clone, Debug, Copy)]
pub enum ComponentType {
    TransformComponent = 0b0000_0000_0000_0001,
    MeshComponent = 0b0000_0000_0000_0010
}

pub type Entity = u32;

pub trait TypeWrapper {
    fn get_type() -> ComponentType;
}

pub struct StorageEntry {
    pub storage_type: u32,
    pub entity: Entity,
    pub component: Component,
}

impl TypeWrapper for StorageEntry {
    fn get_type() -> ComponentType {
        ComponentType::TransformComponent
    }
}
