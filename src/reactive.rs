use std::{
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
    ops::{Deref, DerefMut},
};

/// Thread Safe Reactive Data Structure
/// # Examples
/// ```
/// use reactivate::Reactive;
///
/// let r = Reactive::new("🦀");
/// ```
#[derive(Clone, Default)]
pub struct Reactive<T> {
    #[cfg(not(feature = "threadsafe"))]
    value: std::rc::Rc<std::cell::RefCell<T>>,
    #[cfg(not(feature = "threadsafe"))]
    observers: std::rc::Rc<std::cell::RefCell<Vec<Box<dyn FnMut(&T)>>>>,

    #[cfg(feature = "threadsafe")]
    value: std::sync::Arc<std::sync::Mutex<T>>,
    #[cfg(feature = "threadsafe")]
    observers: std::sync::Arc<std::sync::Mutex<Vec<Box<dyn FnMut(&T) + Send>>>>,
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
            #[cfg(feature = "threadsafe")]
            value: std::sync::Arc::new(std::sync::Mutex::new(value)),

            #[cfg(not(feature = "threadsafe"))]
            value: std::rc::Rc::new(std::cell::RefCell::new(value)),

            observers: Default::default(),
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
        self.acq_val().clone()
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
        f(self.acq_val().deref());
    }

    /// All the Reactive methods acquire and release locks for each method call.
    /// It can be expensive if done repeatedly.
    /// So instead, this method will give mutable access to the internal `value` and `observers`
    /// to do as you please with them.
    ///
    /// Generally not recommended unless you know what you are doing.
    ///
    /// # Examples
    ///
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(10);
    /// r.with(|val, obs| {
    ///     *val += 11;
    ///     for f in obs {
    ///         f(val)
    ///     }
    /// });
    ///
    /// assert_eq!(21, r.value());
    ///
    /// ```
    pub fn with(
        &self,
        #[cfg(not(feature = "threadsafe"))] f: impl FnOnce(&mut T, &mut [Box<dyn FnMut(&T)>]),
        #[cfg(feature = "threadsafe")] f: impl FnOnce(&mut T, &mut [Box<dyn FnMut(&T) + Send>]),
    ) {
        let mut val_guard = self.acq_val();
        let mut obs_guard = self.acq_obs();
        f(val_guard.deref_mut(), obs_guard.deref_mut());
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
    pub fn derive<
        #[cfg(not(feature = "threadsafe"))] U: Clone + PartialEq + 'static,
        #[cfg(feature = "threadsafe")] U: Clone + PartialEq + Send + 'static,
    >(
        &self,
        #[cfg(not(feature = "threadsafe"))] f: impl Fn(&T) -> U + 'static,
        #[cfg(feature = "threadsafe")] f: impl Fn(&T) -> U + Send + 'static,
    ) -> Reactive<U>
    where
        T: Clone,
    {
        let derived_val = f(self.acq_val().deref());
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
    ///
    /// let r = Reactive::new(String::from("🦀"));
    /// r.add_observer(|val| println!("{}", val));
    /// ```
    pub fn add_observer(
        &self,
        #[cfg(not(feature = "threadsafe"))] f: impl FnMut(&T) + 'static,
        #[cfg(feature = "threadsafe")] f: impl FnMut(&T) + Send + 'static,
    ) {
        self.acq_obs().push(Box::new(f));
    }

    /// Clears all observers from the reactive.
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(10);
    /// let d = r.derive(|val| val + 1);
    ///
    /// r.clear_observers();
    /// r.update(|n| n * 2);
    ///
    /// assert_eq!(20, r.value());
    /// // value of `d` didn't change because `r` cleared its observers
    /// assert_eq!(11, d.value());
    /// ```
    pub fn clear_observers(&self) {
        self.acq_obs().clear();
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
    pub fn update_unchecked(&self, f: impl FnOnce(&T) -> T) {
        let mut guard = self.acq_val();
        let val = guard.deref_mut();
        *val = f(val);

        for obs in self.acq_obs().deref_mut() {
            obs(val);
        }
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
    pub fn update_inplace_unchecked(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.acq_val();
        let val = guard.deref_mut();
        f(val);

        for obs in self.acq_obs().deref_mut() {
            obs(val);
        }
    }

    /// Set the value inside the reactive to something new and notify all the observers
    /// by calling the added observer functions in the sequence they were added
    /// (even if the provided value is the same as the current one)
    ///
    /// # Examples
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(10);
    /// let d = r.derive(|val| val + 5);
    ///
    /// r.set(20);
    ///
    /// assert_eq!(25, d.value());
    /// ```
    pub fn set(&self, val: T) {
        let mut guard = self.acq_val();
        let curr_val = guard.deref_mut();
        *curr_val = val;

        for obs in self.acq_obs().deref_mut() {
            obs(curr_val);
        }
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
    pub fn update(&self, f: impl FnOnce(&T) -> T)
    where
        T: PartialEq,
    {
        let mut guard = self.acq_val();
        let val = guard.deref_mut();
        let new_val = f(val);
        if &new_val != val {
            *val = new_val;

            for obs in self.acq_obs().deref_mut() {
                obs(val);
            }
        }
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
    pub fn update_inplace(&self, f: impl FnOnce(&mut T))
    where
        T: Hash,
    {
        let random_state = RandomState::new();

        let mut guard = self.acq_val();
        let val = guard.deref_mut();

        let old_hash = random_state.hash_one(&val);
        f(val);
        let new_hash = random_state.hash_one(&val);

        if old_hash != new_hash {
            for obs in self.acq_obs().deref_mut() {
                obs(val);
            }
        }
    }

    /// Notify all the observers of the current value by calling the
    /// added observer functions in the sequence they were added
    ///
    /// # Examples
    ///
    /// ```
    /// use reactivate::Reactive;
    ///
    /// let r = Reactive::new(String::from("🦀"));
    /// r.add_observer(|val| println!("{}", val));
    /// r.notify();
    /// ```
    pub fn notify(&self) {
        let guard = self.acq_val();
        let val = guard.deref();
        for obs in self.acq_obs().deref_mut() {
            obs(val);
        }
    }

    #[cfg(not(feature = "threadsafe"))]
    fn acq_val(&self) -> std::cell::RefMut<'_, T> {
        self.value.borrow_mut()
    }

    #[cfg(feature = "threadsafe")]
    fn acq_val(&self) -> std::sync::MutexGuard<'_, T> {
        self.value.lock().expect("unable to acquire lock on value")
    }

    #[cfg(not(feature = "threadsafe"))]
    fn acq_obs(&self) -> std::cell::RefMut<'_, Vec<Box<dyn FnMut(&T)>>> {
        self.observers.borrow_mut()
    }

    #[cfg(feature = "threadsafe")]
    fn acq_obs(&self) -> std::sync::MutexGuard<'_, Vec<Box<dyn FnMut(&T) + Send>>> {
        self.observers
            .lock()
            .expect("unable to acquire lock on observers")
    }
}

impl<T: Debug> Debug for Reactive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Reactive")
            .field(self.acq_val().deref())
            .finish()
    }
}
