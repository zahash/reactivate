use reactivate::{Merge, Reactive};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

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
    let changes: Arc<Mutex<Vec<String>>> = Default::default();

    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update(|_| String::from("a"));
    r.update(|_| String::from("a"));
    r.update(|_| String::from("b"));
    r.update(|_| String::from("b"));

    assert_eq!(
        vec![String::from("a"), String::from("b")],
        changes.lock().unwrap().clone()
    );
}

#[test]
fn update_unchecked_notifies_observers_without_checking_if_value_changed() {
    let r: Reactive<String> = Reactive::default();
    let changes: Arc<Mutex<Vec<String>>> = Default::default();

    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update_unchecked(|_| String::from("a"));
    r.update_unchecked(|_| String::from("a"));
    r.update_unchecked(|_| String::from("b"));
    r.update_unchecked(|_| String::from("b"));

    assert_eq!(
        vec![
            String::from("a"),
            String::from("a"),
            String::from("b"),
            String::from("b")
        ],
        changes.lock().unwrap().clone()
    );
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
    let changes: Arc<Mutex<Vec<String>>> = Default::default();

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

    assert_eq!(
        vec![String::from("a"), String::from("ab")],
        changes.lock().unwrap().clone()
    );
}

#[test]
fn update_inplace_unchecked_notifies_observers_without_checking_if_value_changed() {
    let r: Reactive<String> = Reactive::default();
    let changes: Arc<Mutex<Vec<String>>> = Default::default();

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

    assert_eq!(
        vec![String::from("a"), String::from("a"), String::from("ab")],
        changes.lock().unwrap().clone()
    );
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
fn can_clear_observers() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 1);

    r.clear_observers();
    r.update(|n| n * 2);

    assert_eq!(20, r.value());
    assert_eq!(11, d.value());
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
    let record: Arc<Mutex<Vec<String>>> = Default::default();

    r.add_observer({
        let change_log = record.clone();
        move |val| change_log.lock().unwrap().push(val.clone())
    });
    r.add_observer({
        let change_log = record.clone();
        move |_| change_log.lock().unwrap().push(String::from("a"))
    });
    r.add_observer({
        let change_log = record.clone();
        move |_| change_log.lock().unwrap().push(String::from("b"))
    });

    r.notify();
    r.notify();

    assert_eq!(
        vec![
            String::from("🦀"),
            String::from("a"),
            String::from("b"),
            String::from("🦀"),
            String::from("a"),
            String::from("b")
        ],
        record.lock().unwrap().clone()
    );
}

#[test]
#[cfg(feature = "parallel-notification")]
fn can_parallel_notify() {
    let mut r: Reactive<String> = Reactive::new(String::from("zahash"));
    let record: Arc<Mutex<Vec<usize>>> = Default::default();

    r.add_observer({
        let record = record.clone();
        move |s| record.lock().unwrap().push(s.len())
    });
    for i in 10..=100 {
        r.add_observer({
            let record = record.clone();
            move |_| record.lock().unwrap().push(i)
        });
    }

    r.notify();

    // sequential by default
    let mut expected = vec![6];
    expected.extend(10..=100);
    assert_eq!(expected, record.lock().unwrap().clone());

    record.lock().unwrap().clear();

    r.enable_parallel_notification();
    r.notify();

    // the order of calling observers is not sequential.
    let mut result = record.lock().unwrap().clone();
    assert_ne!(expected, result);
    expected.sort();
    result.sort();
    assert_eq!(expected, result);
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
