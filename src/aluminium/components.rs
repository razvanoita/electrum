use cgmath::*;

#[derive(PartialEq, Eq, Hash)]
pub enum ComponentType {
    Transform,
    Mesh,
    Renderable,
}

pub trait Component {
    fn get_type(&self) -> ComponentType;
}

#[derive(Clone, Debug, Copy)]
pub struct Transform {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Vector3<f32>,
    scale: cgmath::Vector3<f32>,
}

impl Component for Transform {
    fn get_type(&self) -> ComponentType {
        ComponentType::Transform
    }
}

