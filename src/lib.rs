//! Thread Safe Reactive Data Structure

mod macros;

use std::{
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

/// Thread Safe Reactive Data Structure using the observer pattern
#[derive(Clone, Default)]
pub struct Reactive<T> {
    inner: Arc<Mutex<ReactiveInner<T>>>,
}

impl<T> Reactive<T> {
    /// Constructs a new Reactive<T>
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new("🦀");
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReactiveInner::new(value))),
        }
    }

    /// Adds a new observer to the reactive.
    /// the observer functions are called whenever the value inside the Reactive is updated
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let r: Reactive<String> = Reactive::default();
    /// // Arc<Mutex<T>> is used to make the vector thread safe
    /// // because Reactive as a whole must be thread safe
    /// let change_log: Arc<Mutex<Vec<String>>> = Default::default();
    ///
    /// // add an observer function to keep a log of all the updates done to the reactive.
    /// r.add_observer({
    ///     let change_log = change_log.clone();
    ///     move |val| change_log.lock().unwrap().push(val.clone())
    /// });
    ///
    /// r.update(|_| String::from("🦀"));
    /// r.update(|_| String::from("🦞"));
    ///
    /// assert_eq!(
    /// vec![String::from("🦀"), String::from("🦞")],
    ///     change_log.lock().unwrap().clone()
    /// );
    /// ```
    pub fn add_observer(&self, f: impl FnMut(&T) + Send + 'static) {
        self.inner.lock().unwrap().observers.push(Box::new(f));
    }

    /// Update the value inside the reactive and notify all the observers
    /// by calling the added observer functions in the sequence they were added
    /// without checking if the value is changed after applying the provided function
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(10);
    /// let d = r.derive(|val| val + 5);
    ///
    /// // notifies the observers as usual because value changed from 10 to 20
    /// r.update_unchecked(|_| 20);
    ///
    /// assert_eq!(25, d.value());
    ///
    /// // would still notify the observers even if the value didn't change
    /// r.update_unchecked(|_| 20);
    ///
    /// assert_eq!(25, d.value());
    /// ```
    ///
    /// # Reasons to use
    /// `update_unchecked` doesn't require `PartialEq` trait bounds on `T`
    /// because the old value and the new value (after applying `f`) aren't compared.
    ///
    /// It is also faster than `update` for that reason
    pub fn update_unchecked(&self, f: impl Fn(&T) -> T) {
        self.inner.lock().unwrap().update_unchecked(f);
    }

    /// Updates the value inside inplace without creating a new clone/copy and notify
    /// all the observers by calling the added observer functions in the sequence they were added
    /// without checking if the value is changed after applying the provided function.
    ///
    /// Perfer this when the datatype inside is expensive to clone, like a vector.
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(vec![1, 2, 3]);
    /// let d = r.derive(|nums| nums.iter().sum::<i32>());
    ///
    /// // notifies the observers as usual because value changed from [1, 2, 3] to [1, 2, 3, 4, 5, 6]
    /// r.update_inplace_unchecked(|nums| {
    ///     nums.push(4);
    ///     nums.push(5);
    ///     nums.push(6);
    /// });
    ///
    /// assert_eq!(21, d.value());
    ///
    /// // would still notify the observers even if the value didn't change
    /// r.update_inplace_unchecked(|nums| {
    ///     nums.push(100);
    ///     nums.pop();
    /// });
    ///
    /// assert_eq!(21, d.value());
    /// ```
    ///
    /// # Reasons to use
    /// `update_inplace_unchecked` doesn't require `Hash` trait bounds on `T`
    /// because the hashes of old value and the new value (after applying `f`)
    /// aren't calculated and compared.
    ///
    /// It is also faster than `update_inplace` for that reason
    pub fn update_inplace_unchecked(&self, f: impl Fn(&mut T)) {
        self.inner.lock().unwrap().update_inplace_unchecked(f);
    }
}

impl<T: Clone> Reactive<T> {
    /// Returns a clone/copy of the value inside the reactive
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(String::from("🦀"));
    /// assert_eq!("🦀", r.value());
    /// ```
    pub fn value(&self) -> T {
        self.inner.lock().unwrap().value.clone()
    }

    /// derive a new child reactive that changes whenever the parent reactive changes.
    /// (achieved by adding an observer function to the parent reactive behind the scenes)
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(10);
    /// let d = r.derive(|val| val + 5);
    ///
    /// assert_eq!(15, d.value());
    /// ```
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
    /// Update the value inside the reactive and notify all the observers
    /// by calling the added observer functions in the sequence they were added
    /// **ONLY** if the value changes after applying the provided function
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(10);
    /// let d = r.derive(|val| val + 5);
    ///
    /// r.update(|_| 20);
    ///
    /// assert_eq!(25, d.value());
    /// ```
    pub fn update(&self, f: impl Fn(&T) -> T) {
        self.inner.lock().unwrap().update(f);
    }
}

impl<T: Hash> Reactive<T> {
    /// Updates the value inside inplace without creating a new clone/copy and notify
    /// all the observers by calling the added observer functions in the sequence they were added
    /// **ONLY** if the value changes after applying the provided function.
    ///
    /// Perfer this when the datatype inside is expensive to clone, like a vector.
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(vec![1, 2, 3]);
    /// let d = r.derive(|nums| nums.iter().sum::<i32>());
    ///
    /// r.update_inplace(|nums| {
    ///     nums.push(4);
    ///     nums.push(5);
    ///     nums.push(6);
    /// });
    ///
    /// assert_eq!(21, d.value());
    /// ```
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

/// This trait is used for implementing variadic generics.
///
/// The main goal is to convert a tuple (can be implemented for other types too)
/// of reactives of arbitrary size
///     `(&Reactive<usize>, &Reactive<String>, &Reactive<f64>, ...)`
///
/// to a tuple of their inner values
///     `(usize, String, f64, ...)`
///
/// Default implementations for tuples is already provided (see `impl_merge_for_tuple` macro)
/// ```
/// use reactivate::{Reactive, Merge};
///
/// let r1: Reactive<usize> = Reactive::default();
/// let r2: Reactive<String> = Reactive::default();
/// let r3: Reactive<f64> = Reactive::default();
///
/// let r: Reactive<(usize, String, f64)> = (&r1, &r2, &r3).merge();
///
/// ```
pub trait Merge {
    type Output;
    fn merge(self) -> Reactive<Self::Output>;
}

/// The purpose of this struct is to reduce boilerplate code when working with `Reactive`
#[derive(Default)]
struct ReactiveInner<T> {
    value: T,
    observers: Vec<Box<dyn FnMut(&T) + Send>>,
}

impl<T> ReactiveInner<T> {
    const fn new(value: T) -> Self {
        Self {
            value,
            observers: vec![],
        }
    }

    fn update_unchecked(&mut self, f: impl Fn(&T) -> T) {
        self.value = f(&self.value);
        for obs in &mut self.observers {
            obs(&self.value);
        }
    }

    fn update_inplace_unchecked(&mut self, f: impl Fn(&mut T)) {
        f(&mut self.value);
        for obs in &mut self.observers {
            obs(&self.value);
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
