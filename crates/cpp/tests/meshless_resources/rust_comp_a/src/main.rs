use a::foo::foo::resources;

mod a;

fn main() {
    {
        let obj = resources::R::new(5);
        obj.add(2);
    }
    let obj2 = resources::create();
    resources::consume(obj2);
}
