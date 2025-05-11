use std::{any::{Any, TypeId}, collections::HashMap};

#[derive(Debug)]
pub struct Anymap(HashMap<TypeId,Box<dyn Any + 'static>>);

impl Anymap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn insert<T: Any>(&mut self, value: T) -> Option<T> {
        self.0.insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|e|Some(*e.downcast().ok()?))
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        self.0.remove(&TypeId::of::<T>())
            .and_then(|e|Some(*e.downcast().ok()?))
    }
}

