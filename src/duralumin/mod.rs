use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::aluminium::components;

type Entity = usize;

pub struct World {
    pending_mask: Option<u32>,
    pending_components: Vec<components::Component>,

    transform_components: HashMap<u32, components::TransformStorage>,
    mesh_components: HashMap<u32, components::MeshStorage>,
}

impl World {
    pub fn new() -> World {
        let free_entities = vec![0];
        World {
            pending_mask: None,
            pending_components: vec![],
            transform_components: HashMap::new(),
            mesh_components: HashMap::new()
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
                self.pending_mask = Some(self.pending_mask.unwrap() | components::ComponentMask::TransformComponent as u32)
            },
            components::Component::MeshComponent(mesh_component) => {
                self.pending_mask = Some(self.pending_mask.unwrap() | components::ComponentMask::MeshComponent as u32)
            },
            _ => println!("Component not supported!")
        }
        self.pending_components.push(component);

        self
    }

    pub fn build(&mut self) {
        assert_eq!(self.pending_mask.is_some(), true);
        assert_eq!(self.pending_components.is_empty(), false);

        while !self.pending_components.is_empty() {
            let component = self.pending_components.pop().unwrap();
            match component {
                components::Component::TransformComponent(transform_component) => {
                    let mut value = self.transform_components.get_mut(&self.pending_mask.unwrap());
                    if value.is_some() {
                        value.unwrap().push(Some(transform_component));
                    } else {
                        self.transform_components.insert(self.pending_mask.unwrap(), vec![Some(transform_component)]);
                    }
                },
                components::Component::MeshComponent(mesh_component) => {
                    let mut value = self.mesh_components.get_mut(&self.pending_mask.unwrap());
                    if value.is_some() {
                        value.unwrap().push(Some(mesh_component));
                    } else {
                        self.mesh_components.insert(self.pending_mask.unwrap(), vec![Some(mesh_component)]);
                    }
                },
                _ => println!("not supported yet!")
            }
        }
    }
}