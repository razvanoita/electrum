use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::aluminium::components;

type Entity = usize;

type Storage = Vec<(Option<components::Component>, Entity)>;

pub struct World {
    pending_mask: Option<u32>,
    pending_components: Vec<components::Component>,

    components: HashMap<u32, Storage>,
}

impl World {
    pub fn new() -> World {
        World {
            pending_mask: None,
            pending_components: vec![],
            components: HashMap::new(),
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

    pub fn build(&mut self) -> Entity {
        assert_eq!(self.pending_mask.is_some(), true);
        assert_eq!(self.pending_components.is_empty(), false);

        let num_components = self.pending_components.len();
        let mut entity: Entity = 0;
        while !self.pending_components.is_empty() {
            let component = self.pending_components.pop().unwrap();
            let mut storage: Option<&mut Storage> = self.components.get_mut(&self.pending_mask.unwrap());
            if storage.is_some() {
                let mut storage_unwrapped: &mut Storage = storage.unwrap();
                entity = (storage_unwrapped.len() / num_components) as usize;
                storage_unwrapped.push((Some(component), entity));
            } else {
                self.components.insert(self.pending_mask.unwrap(), vec![(Some(component), 0)]);
            }
        }
        entity
    }

    pub fn query(&self, mask: u32) {

    }
}