use the_world::exports::foo::foo::strings::Guest;
use the_world::foo::foo::strings;

mod the_world;

struct MyWorld;

impl Guest for MyWorld {
    fn a(x: String) {
        println!("{x}");
    }

    fn b() -> String {
        String::from("hello B")
    }

    fn c(a: String, b: String) -> String {
        println!("{a}|{b}");
        "hello C".into()
    }
}

the_world::export!(MyWorld with_types_in the_world);

fn main() {
    strings::a("hello A");
    {
        let b = strings::b();
        println!("{b}");
    }
    let c = strings::c("hello C1", "hello C2");
    println!("{c}");
}
