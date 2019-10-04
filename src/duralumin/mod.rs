use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::aluminium::components;

pub struct Storage<T: components::Component> {
    storage: HashMap<components::ComponentType, Rc<RefCell<Vec<T>>>>
}

impl<T: components::Component> Storage<T> {
    pub fn register(&mut self, component: T) {
        self.storage.insert(component.get_type(), Rc::new(RefCell::new(vec![])));
    }
}