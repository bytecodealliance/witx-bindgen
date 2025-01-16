use std::sync::Mutex;

use b::exports::foo::foo::resources::{self, Guest, GuestR};

mod b;

b::export!(MyWorld with_types_in b);

#[derive(Debug)]
struct MyResource(Mutex<u32>);

impl GuestR for MyResource {
    fn new(a: u32) -> Self {
        MyResource(Mutex::new(a))
    }

    fn add(&self, b: u32) {
        *self.0.lock().unwrap() += b;
    }
}

struct MyWorld;

impl Guest for MyWorld {
    type R = MyResource;

    fn create() -> resources::R {
        resources::R::new(MyResource::new(17))
    }

    fn consume(o: resources::R) {
        println!(
            "resource consumed with {:?}",
            o.get::<MyResource>().0.lock().unwrap()
        );
    }
}
