use std::sync::atomic::AtomicBool;

wit_bindgen::generate!({
    path: "../tests/runtime/smoke",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

pub struct MyExports;

static HIT: AtomicBool = AtomicBool::new(false);

impl exports::test::smoke::imports::Guest for MyExports {
    fn thunk() {
        HIT.store(true, std::sync::atomic::Ordering::SeqCst);
        println!("tester called");
    }
}

pub fn main() {
    thunk();
    assert!(HIT.load(std::sync::atomic::Ordering::SeqCst));
    {
        #[link(name = "smoke")]
        extern "C" {
            fn thunk();
        }
        let _ = || {
            unsafe { thunk() };
        };
    }
}
