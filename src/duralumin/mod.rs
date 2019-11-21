use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::aluminium::components;

type Entity = usize;

pub struct World {
    pending_mask: Option<u32>
    
    transform_components: HashMap<u32, components::TransformStorage>,
    mesh_components: HashMap<u32, components::MeshStorage>,
}

impl World {
    pub fn new() -> World {
        let free_entities = vec![0];
        World {
            pending_mask: None,
            transform_components: vec![],
            mesh_components: vec![]
        }
    }

    pub fn create_entity(&mut self) -> &mut World {
        self.pending_mask = Some(0);
        self
    }

    pub fn with_component(&mut self, component: components::Component) -> &mut World {
        assert_eq!(self.pending_mask.is_some(), true);

        match component {
            components::Component::TransformComponent(transform_component) => {
                self.transform_components.push(Some(components::TransformStorageEntry {
                    value: transform_component,
                    index: self.pending_entity.unwrap()
                }))
            },
            components::Component::MeshComponent(mesh_component) => {
                self.mesh_components.push(Some(components::MeshStorageEntry {
                    value: mesh_component,
                    index: self.pending_entity.unwrap()
                }))
            }
            _ => println!("not supported yet!")
        }

        self
    }

    pub fn build(&mut self) -> Entity {
        self.pending_entity = None;
        self.entities[self.entities.len() - 1]
    }
}