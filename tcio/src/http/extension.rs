use std::{any::{Any, TypeId}, collections::HashMap};

/// HTTP Extensions.
#[derive(Debug)]
pub struct Extensions(HashMap<TypeId,Box<dyn Any + 'static>>);

impl Extensions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        let ok = self.0.get(&TypeId::of::<T>())?;
        // SAFETY: the provided T is equal to the value
        Some(unsafe { ok.downcast_ref::<T>().unwrap_unchecked() })
    }

    pub fn insert<T: Any>(&mut self, value: T) -> Option<T> {
        let ok = self.0.insert(TypeId::of::<T>(), Box::new(value))?;
        // SAFETY: the provided T is equal to the value
        Some(*unsafe { ok.downcast::<T>().unwrap_unchecked() })
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        let ok = self.0.remove(&TypeId::of::<T>())?;
        // SAFETY: the provided T is equal to the value
        Some(*unsafe { ok.downcast::<T>().unwrap_unchecked() })
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}

