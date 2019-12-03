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

pub struct Velocity {
    pub translation_speed: f32,
    pub rotation_speed: f32,
}

pub enum Component {
    TransformComponent(Transform),
    MeshComponent(Mesh),
    VelocityComponent(Velocity),
}

#[derive(Clone, Debug, Copy)]
pub enum ComponentType {
    TransformComponent = 0b0000_0000_0000_0001,
    MeshComponent = 0b0000_0000_0000_0010,
    VelocityComponent = 0b0000_0000_0000_0100
}

pub type Entity = u32;

pub struct StorageEntry<T> {
    pub storage_type: u32,
    pub entity: Entity,
    pub component: T,
}

pub type TransformStorageEntry = StorageEntry<Transform>;
pub type MeshStorageEntry = StorageEntry<Mesh>;
pub type VelocityStorageEntry = StorageEntry<Velocity>;

