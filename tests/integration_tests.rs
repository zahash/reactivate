use reactivx::{MergeReactive, Reactive};
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

#[test]
fn can_merge() {
    let a = Reactive::new(String::from("hazash"));
    let b = Reactive::new(0isize);
    let d = (&a, &b).merge().derive(|val| val.0.len() as isize + val.1);

    assert_eq!(6, d.value());

    b.update(|_| 5);
    assert_eq!(11, d.value());

    a.update(|_| String::from("mouse"));
    assert_eq!(10, d.value());
}
