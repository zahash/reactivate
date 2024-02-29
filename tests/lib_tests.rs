use reactivate::{Merge, Reactive};

#[test]
fn initial_derived_values_must_not_be_default() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 5);

    assert_eq!(15, d.value());
}

#[test]
fn can_set() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 35);

    r.set(34);

    assert_eq!(34, r.value());
    assert_eq!(69, d.value());
}

#[test]
fn can_update() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 5);

    r.update(|n| n * 2);

    assert_eq!(20, r.value());
    assert_eq!(25, d.value());
}

#[test]
fn update_only_notifies_observers_when_value_changes() {
    let r: Reactive<String> = Reactive::default();

    #[cfg(not(feature = "threadsafe"))]
    let changes: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();

    #[cfg(not(feature = "threadsafe"))]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.borrow_mut().push(val.clone())
    });

    #[cfg(feature = "threadsafe")]
    let changes: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();

    #[cfg(feature = "threadsafe")]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update(|_| String::from("a"));
    r.update(|_| String::from("a"));
    r.update(|_| String::from("b"));
    r.update(|_| String::from("b"));

    let expected = vec![String::from("a"), String::from("b")];

    #[cfg(not(feature = "threadsafe"))]
    assert_eq!(expected, changes.borrow().clone());

    #[cfg(feature = "threadsafe")]
    assert_eq!(expected, changes.lock().unwrap().clone());
}

#[test]
fn update_unchecked_notifies_observers_without_checking_if_value_changed() {
    let r: Reactive<String> = Reactive::default();

    #[cfg(not(feature = "threadsafe"))]
    let changes: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();

    #[cfg(not(feature = "threadsafe"))]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.borrow_mut().push(val.clone())
    });

    #[cfg(feature = "threadsafe")]
    let changes: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();

    #[cfg(feature = "threadsafe")]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update_unchecked(|_| String::from("a"));
    r.update_unchecked(|_| String::from("a"));
    r.update_unchecked(|_| String::from("b"));
    r.update_unchecked(|_| String::from("b"));

    let expected = vec![
        String::from("a"),
        String::from("a"),
        String::from("b"),
        String::from("b"),
    ];

    #[cfg(not(feature = "threadsafe"))]
    assert_eq!(expected, changes.borrow().clone());

    #[cfg(feature = "threadsafe")]
    assert_eq!(expected, changes.lock().unwrap().clone());
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

    assert_eq!(r.value(), vec![1, 2, 3, 4, 5, 6]);
    assert_eq!(21, d.value());
}

#[test]
fn update_inplace_only_notifies_observers_when_value_changes() {
    let r: Reactive<String> = Reactive::default();

    #[cfg(not(feature = "threadsafe"))]
    let changes: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();

    #[cfg(not(feature = "threadsafe"))]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.borrow_mut().push(val.clone())
    });

    #[cfg(feature = "threadsafe")]
    let changes: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();

    #[cfg(feature = "threadsafe")]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update_inplace(|s| s.push('a'));
    r.update_inplace(|s| {
        s.push('x');
        s.pop();
    });
    r.update_inplace(|s| s.push('b'));

    let expected = vec![String::from("a"), String::from("ab")];

    #[cfg(not(feature = "threadsafe"))]
    assert_eq!(expected, changes.borrow().clone());

    #[cfg(feature = "threadsafe")]
    assert_eq!(expected, changes.lock().unwrap().clone());
}

#[test]
fn update_inplace_unchecked_notifies_observers_without_checking_if_value_changed() {
    let r: Reactive<String> = Reactive::default();

    #[cfg(not(feature = "threadsafe"))]
    let changes: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();

    #[cfg(not(feature = "threadsafe"))]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.borrow_mut().push(val.clone())
    });

    #[cfg(feature = "threadsafe")]
    let changes: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();

    #[cfg(feature = "threadsafe")]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update_inplace_unchecked(|s| s.push('a'));
    r.update_inplace_unchecked(|s| {
        s.push('x');
        s.pop();
    });
    r.update_inplace_unchecked(|s| s.push('b'));

    let expected = vec![String::from("a"), String::from("a"), String::from("ab")];

    #[cfg(not(feature = "threadsafe"))]
    assert_eq!(expected, changes.borrow().clone());

    #[cfg(feature = "threadsafe")]
    assert_eq!(expected, changes.lock().unwrap().clone());
}

#[test]
fn can_add_observers() {
    let r: Reactive<String> = Reactive::default();

    #[cfg(not(feature = "threadsafe"))]
    let changes: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();

    #[cfg(not(feature = "threadsafe"))]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.borrow_mut().push(val.clone())
    });

    #[cfg(feature = "threadsafe")]
    let changes: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();

    #[cfg(feature = "threadsafe")]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update(|_| String::from("a"));
    r.update_inplace(|s| {
        s.clear();
        s.push('b');
    });

    let expected = vec![String::from("a"), String::from("b")];

    #[cfg(not(feature = "threadsafe"))]
    assert_eq!(expected, changes.borrow().clone());

    #[cfg(feature = "threadsafe")]
    assert_eq!(expected, changes.lock().unwrap().clone());
}

#[test]
fn can_clear_observers() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 1);

    r.clear_observers();
    r.update(|n| n * 2);

    assert_eq!(20, r.value());
    assert_eq!(11, d.value());
}

#[test]
#[cfg(feature = "threadsafe")]
fn is_threadsafe() {
    let r: Reactive<String> = Reactive::default();

    let handle = std::thread::spawn({
        let r = r.clone();

        move || {
            for _ in 0..10 {
                r.update_inplace(|s| s.push('a'));
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    });

    for _ in 0..10 {
        r.update_inplace(|s| s.push('b'));
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    handle.join().unwrap();

    let value = r.value();
    let num_a = value.matches("a").count();
    let num_b = value.matches("b").count();

    assert_eq!(20, value.len());
    assert_eq!(10, num_a);
    assert_eq!(10, num_b);
}

#[test]
fn can_merge() {
    let a = Reactive::new(String::from("hazash"));
    let b = Reactive::new(0);
    let c = Reactive::new(0.);

    let d = (&a, (&b, &c)).merge();

    assert_eq!((String::from("hazash"), (0, 0.)), d.value());

    a.update(|_| String::from("mouse"));
    assert_eq!((String::from("mouse"), (0, 0.)), d.value());

    b.update(|_| 5);
    assert_eq!((String::from("mouse"), (5, 0.)), d.value());

    c.update(|_| 2.);
    assert_eq!((String::from("mouse"), (5, 2.)), d.value());
}

#[test]
fn can_notify() {
    let r: Reactive<String> = Reactive::new(String::from("🦀"));

    #[cfg(not(feature = "threadsafe"))]
    let changes: std::rc::Rc<std::cell::RefCell<Vec<String>>> = Default::default();

    #[cfg(not(feature = "threadsafe"))]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.borrow_mut().push(val.clone())
    });

    #[cfg(feature = "threadsafe")]
    let changes: std::sync::Arc<std::sync::Mutex<Vec<String>>> = Default::default();

    #[cfg(feature = "threadsafe")]
    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.notify();
    r.notify();
    r.notify();

    let expected = vec![String::from("🦀"), String::from("🦀"), String::from("🦀")];

    #[cfg(not(feature = "threadsafe"))]
    assert_eq!(expected, changes.borrow().clone());

    #[cfg(feature = "threadsafe")]
    assert_eq!(expected, changes.lock().unwrap().clone());
}

#[test]
fn can_access_internals() {
    let r = Reactive::new(10);

    r.with(|val, obs| {
        *val += 11;
        for f in obs {
            f(val)
        }
    });

    assert_eq!(21, r.value());
}
