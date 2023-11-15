<div align="center">

<pre>
██████╗ ███████╗ █████╗  ██████╗████████╗██╗██╗   ██╗ █████╗ ████████╗███████╗
██╔══██╗██╔════╝██╔══██╗██╔════╝╚══██╔══╝██║██║   ██║██╔══██╗╚══██╔══╝██╔════╝
██████╔╝█████╗  ███████║██║        ██║   ██║██║   ██║███████║   ██║   █████╗  
██╔══██╗██╔══╝  ██╔══██║██║        ██║   ██║╚██╗ ██╔╝██╔══██║   ██║   ██╔══╝  
██║  ██║███████╗██║  ██║╚██████╗   ██║   ██║ ╚████╔╝ ██║  ██║   ██║   ███████╗
╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝   ╚═╝   ╚═╝  ╚═══╝  ╚═╝  ╚═╝   ╚═╝   ╚══════╝
------------------------------------------------------------------------------
Thread Safe Reactive Data Structure. Made with ❤️ for 🦀
</pre>

[![Crates.io](https://img.shields.io/crates/v/reactivate.svg)](https://crates.io/crates/reactivate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

## 🚀 Installation

include it in your `Cargo.toml` under `[dependencies]`

```toml
reactivate = { version = "*" }
```

## 🧑‍💻 Usage examples

### 🏗️ Construction

```rust
use reactivate::Reactive;

fn main() {
    let r = Reactive::new(10);

    println!("{:?}", r); // Reactive(10)
    println!("{:?}", r.value()); // 10
}
```

### 🔥 Derive

```rust
use reactivate::Reactive;

fn main() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 5);

    println!("{:?}", r); // Reactive(10)
    println!("{:?}", d); // Reactive(15)
}
```

### ✨ Update

```rust
use reactivate::Reactive;

fn main() {
    let r = Reactive::new(10);
    let d = r.derive(|val| val + 5);

    r.update(|_| 20);

    println!("{:?}", r); // Reactive(20)
    println!("{:?}", d); // Reactive(25)
}
```

### ⚡ Update Inplace

```rust
use reactivate::Reactive;

fn main() {
    let r = Reactive::new(vec![1, 2, 3]);
    let d = r.derive(|nums| nums.iter().sum::<i32>());

    r.update_inplace(|nums| {
        nums.push(4);
        nums.push(5);
        nums.push(6);
    });

    println!("{:?}", r); // Reactive([1, 2, 3, 4, 5, 6])
    println!("{:?}", d); // Reactive(21)
}
```

### 🤝 Merge and Derive

```rust
use reactivate::{Merge, Reactive};

fn main() {
    let a = Reactive::new(String::from("hazash"));
    let b = Reactive::new(0);
    let d = (&a, &b)
        .merge()
        .derive(|(a_val, b_val)| a_val.len() + b_val);

    println!("{:?}", a); // Reactive("hazash")
    println!("{:?}", b); // Reactive(0)
    println!("{:?}", d); // Reactive(6)

    b.update(|_| 5);

    println!("{:?}", a); // Reactive("hazash")
    println!("{:?}", b); // Reactive(5)
    println!("{:?}", d); // Reactive(11)


    a.update(|_| String::from("mouse"));

    println!("{:?}", a); // Reactive("mouse")
    println!("{:?}", b); // Reactive(5)
    println!("{:?}", d); // Reactive(10)
}
```

### 👀 Add Observers

```rust
use reactivate::Reactive;
use std::sync::{Arc, Mutex};

fn main() {
    let r: Reactive<String> = Reactive::default();

    // Arc<Mutex<T>> is used to make the vector thread safe
    // because Reactive as a whole must be thread safe
    let changes: Arc<Mutex<Vec<String>>> = Default::default();

    r.add_observer({
        let changes = changes.clone();
        move |val| changes.lock().unwrap().push(val.clone())
    });

    r.update(|_| String::from("a"));
    r.update_inplace(|s| {
        s.push('b');
    });

    println!("{:?}", r); // Reactive("ab")
    println!("{:?}", changes.lock().unwrap().clone()); // ["a", "ab"]
}
```

### 🧵 With Threads

```rust
use reactivate::Reactive;
use std::{thread, time::Duration};

fn main() {
    let r: Reactive<String> = Reactive::default();
    let d = r.derive(|s| s.len());

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

    println!("{:?}", r); // Reactive("babababababababababa")
    println!("{:?}", d); // Reactive(20)
}
```

## 🌟 Connect with Us

M. Zahash – zahash.z@gmail.com

Distributed under the MIT license. See `LICENSE` for more information.

[https://github.com/zahash/](https://github.com/zahash/)

## 🤝 Contribute to Reactivate!

1. Fork it (<https://github.com/zahash/reactivate/fork>)
2. Create your feature branch (`git checkout -b feature/fooBar`)
3. Commit your changes (`git commit -am 'Add some fooBar'`)
4. Push to the branch (`git push origin feature/fooBar`)
5. Create a new Pull Request

## ❤️ Show Some Love!

If you find Reactivate helpful and enjoy using it, consider giving it a [⭐ on GitHub!](https://github.com/zahash/reactivate/stargazers) Your star is a gesture of appreciation and encouragement for the continuous improvement of Reactivate.
