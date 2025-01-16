mod w;

struct MyImpl;

impl w::exports::test::test::i::Guest for MyImpl {
    fn f(a: Vec<String>) -> Vec<String> {
        a
    }

    fn g(a: Vec<u8>) -> Vec<u8> {
        a
    }
}

w::export!(MyImpl with_types_in w);
