//! Thread Safe Reactive Data Structure

use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub struct Reactive<T> {
    inner: Arc<Mutex<ReactiveInner<T>>>,
}

impl<T> Reactive<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReactiveInner::new(value))),
        }
    }

    pub fn add_observer(&self, f: impl FnMut(&T) + Send + 'static) {
        self.inner.lock().unwrap().observers.push(Box::new(f));
    }
}

impl<T: Clone> Reactive<T> {
    pub fn value(&self) -> T {
        self.inner.lock().unwrap().value.clone()
    }

    pub fn derive<U: Default + Clone + PartialEq + Send + 'static>(
        &self,
        f: impl Fn(&T) -> U + Send + 'static,
    ) -> Reactive<U> {
        let derived_val = f(&self.value());
        let derived: Reactive<U> = Reactive::new(derived_val);

        self.add_observer({
            let derived = derived.clone();
            move |value| derived.update(|_| f(value))
        });

        derived
    }
}

impl<T: PartialEq> Reactive<T> {
    pub fn update(&self, f: impl Fn(&T) -> T) {
        self.inner.lock().unwrap().update(f);
    }
}

impl<T: Hash> Reactive<T> {
    pub fn update_inplace(&self, f: impl Fn(&mut T)) {
        self.inner.lock().unwrap().update_inplace(f);
    }
}

impl<T: Debug> Debug for Reactive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Reactive")
            .field(&self.inner.lock().unwrap().value)
            .finish()
    }
}

#[derive(Clone)]
pub struct ReactiveBuilder<S> {
    reactive: Reactive<S>,
}

impl<S: Default> ReactiveBuilder<S> {
    pub fn new() -> Self {
        Self {
            reactive: Reactive::default(),
        }
    }
}

impl<S: Clone + Hash + Send + 'static> ReactiveBuilder<S> {
    pub fn add<T: Clone>(self, r: &Reactive<T>, f: impl Fn((&mut S, &T)) + Send + 'static) -> Self {
        self.reactive.update_inplace(|val| f((val, &r.value())));

        r.add_observer({
            let this = self.reactive.clone();
            move |r_val| this.update_inplace(|this_val| f((this_val, r_val)))
        });

        self
    }
}

impl<S> ReactiveBuilder<S> {
    pub fn build(self) -> Reactive<S> {
        self.reactive
    }
}

// variadic generics in rust

pub trait MergeReactive: Sized {
    type Output;
    fn merge(self) -> Reactive<Self::Output>;
}

impl<T0: Clone + Default + Hash + Send + 'static, T1: Clone + Default + Hash + Send + 'static>
    MergeReactive for (&Reactive<T0>, &Reactive<T1>)
{
    type Output = (T0, T1);

    fn merge(self) -> Reactive<Self::Output> {
        let unwrapped = (self.0.value(), self.1.value());
        let combined = Reactive::new(unwrapped);

        self.0.add_observer({
            let combined = combined.clone();
            move |val| combined.update_inplace(|c| c.0 = val.clone())
        });

        self.1.add_observer({
            let combined = combined.clone();
            move |val| combined.update_inplace(|c| c.1 = val.clone())
        });

        combined
    }
}

#[derive(Default)]
struct ReactiveInner<T> {
    value: T,
    observers: Vec<Box<dyn FnMut(&T) + Send>>,
}

impl<T> ReactiveInner<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            observers: vec![],
        }
    }
}

impl<T: PartialEq> ReactiveInner<T> {
    fn update(&mut self, f: impl Fn(&T) -> T) {
        let new_value = f(&self.value);
        if new_value != self.value {
            self.value = new_value;
            for obs in &mut self.observers {
                obs(&self.value);
            }
        }
    }
}

impl<T: Hash> ReactiveInner<T> {
    fn update_inplace(&mut self, f: impl Fn(&mut T)) {
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
            for obs in &mut self.observers {
                obs(&self.value);
            }
        }
    }
}
