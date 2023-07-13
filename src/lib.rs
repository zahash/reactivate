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

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::*;

    #[test]
    fn initial_derived_values_must_not_be_default() {
        let r = Reactive::new(10);
        let d = r.derive(|val| val + 5);

        assert_eq!(15, d.value());
    }

    #[test]
    fn can_update() {
        let r = Reactive::new(10);
        let d = r.derive(|val| val + 5);

        r.update(|_| 20);

        assert_eq!(25, d.value());
    }

    #[test]
    fn can_update_inplace() {
        let r = Reactive::new(vec![1, 2, 3]);
        let d = r.derive(|nums| nums.iter().sum::<i32>());

        r.update_inplace(|nums| {
            nums.push(4);
            nums.push(5);
            nums.push(6);
        });

        assert_eq!(21, d.value());
    }

    #[test]
    fn can_add_observers() {
        let r: Reactive<String> = Reactive::default();
        let changes: Arc<Mutex<Vec<String>>> = Default::default();

        r.add_observer({
            let changes = changes.clone();
            move |val| changes.lock().unwrap().push(val.clone())
        });

        r.update(|_| String::from("a"));
        r.update_inplace(|s| {
            s.clear();
            s.push('b');
        });

        assert_eq!(
            vec![String::from("a"), String::from("b")],
            changes.lock().unwrap().clone()
        );
    }

    #[test]
    fn is_threadsafe() {
        let r: Reactive<String> = Reactive::default();

        let handle = thread::spawn({
            let r = r.clone();

            move || {
                for _ in 0..10 {
                    r.update_inplace(|s| s.push('a'));
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });

        for _ in 0..10 {
            r.update_inplace(|s| s.push('b'));
            thread::sleep(Duration::from_millis(1));
        }

        handle.join().unwrap();

        let value = r.value();
        let num_a = value.matches("a").count();
        let num_b = value.matches("b").count();

        assert_eq!(20, value.len());
        assert_eq!(10, num_a);
        assert_eq!(10, num_b);
    }
}
