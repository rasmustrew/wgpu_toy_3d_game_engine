use std::{rc::{Rc}};

pub struct World {
    entity_counter: u64,
}

pub struct Entity {
    _id: u64,
    pub components: Vec<Component>
}

pub enum Component {
    Model(Rc<crate::model::Model>),
    Instance(crate::instance::Instance, wgpu::Buffer)
}


impl World {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
        }
    }

    pub fn create_entity (&mut self, components: Vec<Component>) -> Entity {
        self.entity_counter += 1;
        Entity {
            _id: self.entity_counter,
            components,
        }
    }
    
}