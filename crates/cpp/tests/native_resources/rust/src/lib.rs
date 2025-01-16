use the_world::exports::foo::foo::resources::{self, Guest, GuestR, RBorrow};
use core::alloc::Layout;
use std::sync::Mutex;

mod the_world;

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
        resources::R::new(MyResource::new(1))
    }
    fn borrows(o: RBorrow<'_>) {
        println!("resource borrowed with {:?}", o.get::<MyResource>().0.lock().unwrap());
    }
    fn consume(o: resources::R) {
        println!("resource consumed with {:?}", o.get::<MyResource>().0.lock().unwrap());

        println!("exercise the other direction");
        let obj = the_world::foo::foo::resources::create();
        obj.add(12);
        the_world::foo::foo::resources::borrows(&obj);
        the_world::foo::foo::resources::consume(obj);
        let obj2 = the_world::foo::foo::resources::R::new(42);
        drop(obj2);
    }
}

the_world::export!(MyWorld with_types_in the_world);
