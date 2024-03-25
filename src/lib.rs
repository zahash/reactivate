//! Thread Safe Reactive Data Structure. Made with ❤️ for 🦀
//!
//! # Reactive
//!
//! `Reactive` is a thread-safe reactive data structure for managing state changes
//! in single-threaded or multi-threaded environments. It allows users to create reactive values that automatically
//! notifies observers when their underlying values change.
//!
//! ## Update
//!
//! ```
//! use reactivate::Reactive;
//!
//! let r = Reactive::new(10);
//! let d = r.derive(|val| val + 5);
//!
//! r.update(|val| val * 2);
//! r.update_inplace(|val| *val += 1);
//! r.update_unchecked(|val| val * 2);
//! r.update_inplace_unchecked(|val| *val += 1);
//!
//! println!("{:?}", r); // Reactive(43)
//! println!("{:?}", d); // Reactive(48)
//! ```
//!
//! ## Observers
//!
//! ```
//! use reactivate::Reactive;
//! # use std::rc::Rc;
//! # use std::cell::RefCell;
//! # use std::sync::{Arc, Mutex};
//!
//! let r = Reactive::new(0);
//!
//! // normal observer
//! r.add_observer(|val| println!("{}", val));
//!
//! // non-threadsafe observer
//! let changes: Rc<RefCell<Vec<usize>>> = Default::default();
//! r.add_observer({
//!     let changes = changes.clone();
//!     move |val| changes.borrow_mut().push(*val)
//! });
//!
//! // threadsafe observer
//! // must use Arc<Mutex<Vec<_>>> when threadsafety is enabled (features = ["threadsafe"])
//! // because Reactive as a whole must be thread safe
//! let tsafe_changes: Arc<Mutex<Vec<usize>>> = Default::default();
//! r.add_observer({
//!     let changes = tsafe_changes.clone();
//!     move |val| changes.lock().expect("unable to acq lock").push(*val)
//! });
//! ```
//!
//! ## Merge
//!
//! ```
//! use reactivate::{Merge, Reactive};
//!
//! let a = Reactive::new(String::from("hazash"));
//! let b = Reactive::new(0);
//! let d = (&a, &b).merge().derive(|(s, n)| s.len() + n);
//!
//! println!("{:?}", a); // Reactive("hazash")
//! println!("{:?}", b); // Reactive(0)
//! println!("{:?}", d); // Reactive(6)
//!
//! b.update(|_| 5);
//!
//! println!("{:?}", a); // Reactive("hazash")
//! println!("{:?}", b); // Reactive(5)
//! println!("{:?}", d); // Reactive(11)
//!
//!
//! a.update(|_| String::from("mouse"));
//!
//! println!("{:?}", a); // Reactive("mouse")
//! println!("{:?}", b); // Reactive(5)
//! println!("{:?}", d); // Reactive(10)
//! ```
//!
//! ## With Threads (features = ["threadsafe"])
//!
//! ```
//! # #[cfg(feature = "threadsafe")]
//! # {
//! use reactivate::Reactive;
//! use std::{thread, time::Duration};
//!
//! let r: Reactive<String> = Reactive::default();
//! let d = r.derive(|s| s.len());
//!
//! let handle = thread::spawn({
//!     let r = r.clone();
//!
//!     move || {
//!         for _ in 0..10 {
//!             r.update_inplace(|s| s.push('a'));
//!             thread::sleep(Duration::from_millis(1));
//!         }
//!     }
//! });
//!
//! for _ in 0..10 {
//!     r.update_inplace(|s| s.push('b'));
//!     thread::sleep(Duration::from_millis(1));
//! }
//!
//! handle.join().unwrap();
//!
//! println!("{:?}", r); // Reactive("babababababababababa")
//! println!("{:?}", d); // Reactive(20)
//! # }
//! ```
//!
//! ## Concurrency
//!
//! `Reactive` provides thread-safe implementations using `Arc` and `Mutex` for multi-threaded environments. Ensure to enable the `threadsafe` feature to use the thread-safe version.
//!
//! ## Performance
//!
//! For performance-critical scenarios, `Reactive` provides methods like `update_unchecked` and `update_inplace_unchecked` for efficient updates without checking for value changes, optimizing performance especially in cases where frequent updates occur.
//!
//! For more details and usage examples, refer to the individual method documentations.
//!

mod macros;
mod merge;
mod reactive;

pub use merge::Merge;
pub use reactive::Reactive;
