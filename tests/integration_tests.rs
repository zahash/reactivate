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
fn can_update() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 5);

    r.update(|_| 20);

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
    let d = (&a, &b)
        .merge()
        .derive(|(a_val, b_val)| a_val.len() + b_val);

    assert_eq!(6, d.value());

    b.update(|_| 5);
    assert_eq!(11, d.value());

    a.update(|_| String::from("mouse"));
    assert_eq!(10, d.value());
}
