use crate::base::ReactiveBase;

use std::{
    fmt::Debug,
    hash::Hash,
    ops::DerefMut,
    sync::{Arc, Mutex, MutexGuard},
};

/// Thread Safe Reactive Data Structure using the observer pattern
#[derive(Clone, Default)]
pub struct Reactive<T> {
    inner: Arc<Mutex<ReactiveBase<T>>>,
}

impl<T> Reactive<T> {
    /// Constructs a new Reactive<T>
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new("🦀");
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReactiveBase::new(value))),
        }
    }

    /// Returns a clone/copy of the value inside the reactive
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(String::from("🦀"));
    /// assert_eq!("🦀", r.value());
    /// ```
    pub fn value(&self) -> T
    where
        T: Clone,
    {
        self.acq_lock().value().clone()
    }

    /// Perform some action with the reference to the inner value.
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(String::from("🦀"));
    /// r.with_value(|s| println!("{}", s));
    /// ```
    pub fn with_value(&self, f: impl FnOnce(&T)) {
        self.acq_lock().with_value(f);
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
    pub fn derive<U>(&self, f: impl Fn(&T) -> U + Send + 'static) -> Reactive<U>
    where
        T: Clone,
        U: Default + Clone + PartialEq + Send + 'static,
    {
        let derived_val = f(&self.value());
        let derived: Reactive<U> = Reactive::new(derived_val);

        self.add_observer({
            let derived = derived.clone();
            move |value| derived.update(|_| f(value))
        });

        derived
    }

    /// Adds a new observer to the reactive.
    /// the observer functions are called whenever the value inside the Reactive is updated
    ///
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
        self.acq_lock().add_observer(f);
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
        self.acq_lock().update_unchecked(f);
    }

    /// Updates the value inside inplace without creating a new clone/copy and notify
    /// all the observers by calling the added observer functions in the sequence they were added
    /// without checking if the value is changed after applying the provided function.
    ///
    /// Prefer this when the datatype inside is expensive to clone, like a vector.
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
        self.acq_lock().update_inplace_unchecked(f);
    }

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
    pub fn update(&self, f: impl Fn(&T) -> T)
    where
        T: PartialEq,
    {
        self.acq_lock().update(f);
    }

    /// Updates the value inside inplace without creating a new clone/copy and notify
    /// all the observers by calling the added observer functions in the sequence they were added
    /// **ONLY** if the value changes after applying the provided function.
    ///
    /// Prefer this when the datatype inside is expensive to clone, like a vector.
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
    pub fn update_inplace(&self, f: impl Fn(&mut T))
    where
        T: Hash,
    {
        self.acq_lock().update_inplace(f);
    }

    /// Notify all the observers of the current value by calling the
    /// added observer functions in the sequence they were added
    ///
    /// # Examples
    ///
    /// ```
    /// use reactivate::Reactive;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let r: Reactive<String> = Reactive::new(String::from("🦀"));
    /// let change_log: Arc<Mutex<Vec<String>>> = Default::default();
    ///
    /// r.add_observer({
    ///     let change_log = change_log.clone();
    ///     move |val| change_log.lock().unwrap().push(val.clone())
    /// });
    ///
    /// r.notify();
    /// r.notify();
    /// r.notify();
    ///
    /// assert_eq!(
    /// vec![String::from("🦀"), String::from("🦀"), String::from("🦀"),],
    ///     change_log.lock().unwrap().clone()
    /// );
    /// ```
    pub fn notify(&self) {
        self.acq_lock().notify();
    }

    /// All the Reactive methods acquire and release locks for each method call.
    /// It can be expensive if done repeatedly.
    /// So instead, this method will give mutable access to the inner `ReactiveBase`
    /// to allow calling multiple methods with a single lock
    ///
    /// # Examples
    ///
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(10);
    /// r.with_inner(|inner| {
    ///     inner.update(|v| v + 10);
    ///     inner.update_inplace(|v| *v += 1);
    ///     inner.notify();
    /// });
    ///
    /// assert_eq!(21, r.value());
    ///
    /// ```
    pub fn with_inner(&self, f: impl FnOnce(&mut ReactiveBase<T>)) {
        f(self.acq_lock().deref_mut());
    }

    fn acq_lock(&self) -> MutexGuard<'_, ReactiveBase<T>> {
        self.inner.lock().unwrap()
    }
}

impl<T: Debug> Debug for Reactive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Reactive")
            .field(self.acq_lock().value())
            .finish()
    }
}
