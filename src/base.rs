use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Deref,
};

#[derive(Default)]
pub struct ReactiveBase<T> {
    value: T,
    observers: Vec<Box<dyn FnMut(&T) + Send>>,
}

impl<T> ReactiveBase<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value,
            observers: vec![],
        }
    }

    pub const fn value(&self) -> &T {
        &self.value
    }

    pub fn with_value(&self, f: impl FnOnce(&T)) {
        f(&self.value);
    }

    pub fn add_observer(&mut self, f: impl FnMut(&T) + Send + 'static) {
        self.observers.push(Box::new(f));
    }

    pub fn update_unchecked(&mut self, f: impl Fn(&T) -> T) {
        self.value = f(&self.value);
        self.notify();
    }

    pub fn update_inplace_unchecked(&mut self, f: impl Fn(&mut T)) {
        f(&mut self.value);
        self.notify();
    }

    pub fn update(&mut self, f: impl Fn(&T) -> T)
    where
        T: PartialEq,
    {
        let new_value = f(&self.value);
        if new_value != self.value {
            self.value = new_value;
            self.notify();
        }
    }

    pub fn update_inplace(&mut self, f: impl Fn(&mut T))
    where
        T: Hash,
    {
        let old_hash = {
            let mut hasher = DefaultHasher::new();
            self.value.hash(&mut hasher);
            hasher.finish()
        };

        f(&mut self.value);

        let new_hash = {
            let mut hasher = DefaultHasher::new();
            self.value.hash(&mut hasher);
            hasher.finish()
        };

        if old_hash != new_hash {
            self.notify();
        }
    }

    pub fn notify(&mut self) {
        for obs in &mut self.observers {
            obs(&self.value);
        }
    }
}

impl<T> Deref for ReactiveBase<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
