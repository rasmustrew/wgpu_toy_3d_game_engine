use std::{rc::{Rc}};

pub struct World {
    entity_counter: u64,
    pub entities: Vec<Entity>,
}

pub struct Entity {
    _id: u64,
    pub components: Vec<Component>
}

pub enum Component {
    Model(Rc<crate::model::Model>),
    Transform(crate::transform::Transform, wgpu::Buffer),
}


impl World {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
            entities: vec![],
        }
    }

    pub fn create_entity (&mut self, components: Vec<Component>) {
        self.entity_counter += 1;
        let entity = Entity {
            _id: self.entity_counter,
            components,
        };
        self.entities.push(entity);
    }

    pub fn act_on_components<F>(&mut self, f: &mut F) where
    F: FnMut(&mut Component) -> () {
        self.entities.iter_mut().for_each(|entity|{
            entity.components.iter_mut().for_each(|component| {
                f(component)
            }); 
        });
    }

    pub fn do_with_components<F>(&self, f: F) where
    F: Fn(&Component) -> () {
        self.entities.iter().for_each(|entity|{
            entity.components.iter().for_each(|component| {
                f(component)
            }); 
        });
    }

    
}