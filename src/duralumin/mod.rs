use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::slice::Iter;
use std::slice::IterMut;
use std::iter::Iterator;
use std::ops::DerefMut;

use crate::aluminium::components;

pub struct World {
    pending_mask: Option<u32>,
    pending_components: Vec<components::Component>,
    last_build_entity: Option<components::Entity>,

    transform_storage: Vec<components::StorageEntry>,
    mesh_storage: Vec<components::StorageEntry>
}

impl World {
    pub fn new() -> World {
        World {
            pending_mask: None,
            pending_components: vec![],
            last_build_entity: None,
            transform_storage: vec![],
            mesh_storage: vec![]
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
                components::Component::TransformComponent(_) => {
                   let entry = components::StorageEntry {
                       storage_type: storage_type,
                       entity: entity,
                       component: component
                   };
                   self.transform_storage.push(entry);
                },
                components::Component::MeshComponent(_) => {
                    let entry = components::StorageEntry {
                        storage_type: storage_type,
                        entity: entity,
                        component: component
                    };
                    self.mesh_storage.push(entry);
                },
                _ => println!("Component not supported!")
            }

        }

        self.pending_mask = None;
        self.pending_components.clear();
        self.last_build_entity = Some(entity);

        entity
    }

    fn get_storage_mut(&mut self, t: components::ComponentType) -> &mut Vec<components::StorageEntry> {
        match t {
            components::ComponentType::TransformComponent => &mut self.transform_storage,
            components::ComponentType::MeshComponent => &mut self.mesh_storage
        }
    }

    fn get_storage(&self, t: components::ComponentType) -> &Vec<components::StorageEntry> {
        match t {
            components::ComponentType::TransformComponent => &self.transform_storage,
            components::ComponentType::MeshComponent => &self.mesh_storage
        }
    }

    pub fn query_1(&mut self, t0: components::ComponentType) -> Iter<components::StorageEntry> {
        match t0 {
            components::ComponentType::TransformComponent => self.transform_storage.iter(),
            components::ComponentType::MeshComponent => self.mesh_storage.iter()
        }
    }

    pub fn query_2(
        &mut self, 
        t0: components::ComponentType, 
        t1: components::ComponentType
    ) -> Vec<(&components::StorageEntry, &mut components::StorageEntry)> {
        let mask: u32 = (t0 as u32) | (t1 as u32);

        let f0: Vec<&components::StorageEntry> = self.transform_storage.iter()
            .filter(|entry| {
                (entry.storage_type as u32) & &mask == mask
            })
            .collect();

        let f1: Vec<&mut components::StorageEntry> = self.mesh_storage.iter_mut()
            .filter(|entry| {
                (entry.storage_type as u32) & &mask == mask
            })
            .collect();

        let res = f0.iter().zip(f1.iter()).map(|(i0, i1)| (*i0, *i1)).collect();
        res
    }
}