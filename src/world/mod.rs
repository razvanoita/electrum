use std::cell::RefCell;
use std::rc::Rc;
use std::slice::Iter;
use std::slice::IterMut;
use std::iter::Iterator;
use std::ops::DerefMut;

use crate::components;

pub struct World {
    pending_mask: Option<u32>,
    pending_components: Vec<components::Component>,
    last_build_entity: Option<components::Entity>,

    pub transform_storage: Vec<components::TransformStorageEntry>,
    pub mesh_storage: Vec<components::MeshStorageEntry>,
    pub velocity_storage: Vec<components::VelocityStorageEntry>,
    pub material_storage: Vec<components::MaterialStorageEntry>,
}

impl World {
    pub fn new() -> World {
        World {
            pending_mask: None,
            pending_components: vec![],
            last_build_entity: None,
            transform_storage: vec![],
            mesh_storage: vec![],
            velocity_storage: vec![],
            material_storage: vec![],
        }
    }

    pub fn create_entity(&mut self) -> &mut World {
        assert_eq!(self.pending_components.is_empty(), true);

        self.pending_mask = Some(0);
        self
    }

    pub fn with_component(&mut self, component: components::Component) -> &mut World {
        assert_eq!(self.pending_mask.is_some(), true);
       
        match &component {
            components::Component::TransformComponent(transform_component) => {
                self.pending_mask = Some(self.pending_mask.unwrap() | components::ComponentType::TransformComponent as u32)
            },
            components::Component::MeshComponent(mesh_component) => {
                self.pending_mask = Some(self.pending_mask.unwrap() | components::ComponentType::MeshComponent as u32)
            },
            components::Component::VelocityComponent(velocity_component) => {
                self.pending_mask = Some(self.pending_mask.unwrap() | components::ComponentType::VelocityComponent as u32)
            },
            components::Component::MaterialComponent(material_component) => {
                self.pending_mask = Some(self.pending_mask.unwrap() | components::ComponentType::MaterialComponent as u32)
            },
            _ => println!("Component not supported!")
        }
        self.pending_components.push(component);

        self
    }

    pub fn build(&mut self) -> components::Entity {
        assert_eq!(self.pending_mask.is_some(), true);
        assert_eq!(self.pending_components.is_empty(), false);

        let storage_type = self.pending_mask.unwrap();
        let entity = match self.last_build_entity {
            None => 0,
            Some(x) => (x + 1) as components::Entity
        };
        while !self.pending_components.is_empty() {
            let component = self.pending_components.pop().unwrap();
            
            match component {
                components::Component::TransformComponent(transform) => {
                   let entry = components::StorageEntry::<components::Transform> {
                       storage_type: storage_type,
                       entity: entity,
                       component: transform
                   };
                   self.transform_storage.push(entry);
                },
                components::Component::MeshComponent(mesh) => {
                    let entry = components::StorageEntry::<components::Mesh> {
                        storage_type: storage_type,
                        entity: entity,
                        component: mesh
                    };
                    self.mesh_storage.push(entry);
                },
                components::Component::VelocityComponent(velocity) => {
                    let entry = components::StorageEntry::<components::Velocity> {
                        storage_type: storage_type,
                        entity: entity,
                        component: velocity
                    };
                    self.velocity_storage.push(entry);
                },
                components::Component::MaterialComponent(material) => {
                    let entry = components::StorageEntry::<components::Material> {
                        storage_type: storage_type,
                        entity: entity,
                        component: material
                    };
                    self.material_storage.push(entry);
                },
                _ => println!("Component not supported!")
            }

        }

        self.pending_mask = None;
        self.pending_components.clear();
        self.last_build_entity = Some(entity);

        entity
    }
}