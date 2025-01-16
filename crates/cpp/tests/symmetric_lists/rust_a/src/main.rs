// compile with: RUSTFLAGS=-L../rust_b/target/debug cargo build

mod x;

// force linking to librust_b.so
#[allow(dead_code)]
fn b() {
    #[link(name = "rust_b")]
    extern "C" {
        fn testX3AtestX2FiX00f(_: *mut u8, _: usize, _: *mut u8);
    }
    unsafe { testX3AtestX2FiX00f(core::ptr::null_mut(), 0, core::ptr::null_mut()) };
}

fn main() {
    let input = vec!["hello".into(), "world".into()];
    let output = x::test::test::i::f(&input);
    println!("{output:?}");
    let input2 = vec![1,2,3];
    let output2 = x::test::test::i::g(&input2);
    println!("{output2:?}");
}
