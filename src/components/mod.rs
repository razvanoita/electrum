#[macro_use]

use cgmath::*;

use crate::render;
use ash::vk;

pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>,
    pub scale: cgmath::Vector3<f32>,
}

pub struct Mesh {
    pub vertex_buffer: render::buffer::VertexBuffer,
    pub index_buffer: render::buffer::IndexBuffer,
}

pub struct Velocity {
    pub translation_speed: f32,
    pub rotation_speed: f32,
}

pub struct Material {
    pub vertex_shader: vk::ShaderModule,
    pub fragment_shader: vk::ShaderModule,
    pub pso: vk::Pipeline,
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub color_blend_attachment_states: Vec<vk::PipelineColorBlendAttachmentState>
}

pub enum Component {
    TransformComponent(Transform),
    MeshComponent(Mesh),
    VelocityComponent(Velocity),
    MaterialComponent(Material),
}

#[derive(Clone, Debug, Copy)]
pub enum ComponentType {
    TransformComponent = 0b0000_0000_0000_0001,
    MeshComponent = 0b0000_0000_0000_0010,
    VelocityComponent = 0b0000_0000_0000_0100,
    MaterialComponent = 0b0000_0000_0000_1000
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
pub type MaterialStorageEntry = StorageEntry<Material>;

