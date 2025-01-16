// checking balanced memory can't work in symmetric because ownership is transferred both ways

pub fn get() -> usize {
    0
}

pub fn guard() -> impl Drop {
    struct A;

    impl Drop for A {
        fn drop(&mut self) {}
    }

    A
}
