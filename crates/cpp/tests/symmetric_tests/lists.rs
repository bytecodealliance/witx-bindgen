wit_bindgen::generate!({
    path: "../tests/runtime/lists",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

pub struct MyExports;

impl exports::test::lists::test::Guest for MyExports {
    fn empty_list_param(a: Vec<u8>) {
        assert!(a.is_empty());
    }

    fn empty_string_param(a: String) {
        assert_eq!(a, "");
    }

    fn empty_list_result() -> Vec<u8> {
        Vec::new()
    }

    fn empty_string_result() -> String {
        String::new()
    }

    fn list_param(list: Vec<u8>) {
        assert_eq!(list, [1, 2, 3, 4]);
    }

    fn list_param2(ptr: String) {
        assert_eq!(ptr, "foo");
    }

    fn list_param3(ptr: Vec<String>) {
        assert_eq!(ptr.len(), 3);
        assert_eq!(ptr[0], "foo");
        assert_eq!(ptr[1], "bar");
        assert_eq!(ptr[2], "baz");
    }

    fn list_param4(ptr: Vec<Vec<String>>) {
        assert_eq!(ptr.len(), 2);
        assert_eq!(ptr[0][0], "foo");
        assert_eq!(ptr[0][1], "bar");
        assert_eq!(ptr[1][0], "baz");
    }

    fn list_result() -> Vec<u8> {
        vec![1, 2, 3, 4, 5]
    }

    fn list_result2() -> String {
        "hello!".to_string()
    }

    fn list_result3() -> Vec<String> {
        vec!["hello,".to_string(), "world!".to_string()]
    }

    fn list_roundtrip(list: Vec<u8>) -> Vec<u8> {
        list.to_vec()
    }

    fn string_roundtrip(s: String) -> String {
        s.to_string()
    }

    fn list_minmax8(u: Vec<u8>, s: Vec<i8>) -> (Vec<u8>, Vec<i8>) {
        assert_eq!(u, [u8::MIN, u8::MAX]);
        assert_eq!(s, [i8::MIN, i8::MAX]);
        (u, s)
    }

    fn list_minmax16(u: Vec<u16>, s: Vec<i16>) -> (Vec<u16>, Vec<i16>) {
        assert_eq!(u, [u16::MIN, u16::MAX]);
        assert_eq!(s, [i16::MIN, i16::MAX]);
        (u, s)
    }

    fn list_minmax32(u: Vec<u32>, s: Vec<i32>) -> (Vec<u32>, Vec<i32>) {
        assert_eq!(u, [u32::MIN, u32::MAX]);
        assert_eq!(s, [i32::MIN, i32::MAX]);
        (u, s)
    }

    fn list_minmax64(u: Vec<u64>, s: Vec<i64>) -> (Vec<u64>, Vec<i64>) {
        assert_eq!(u, [u64::MIN, u64::MAX]);
        assert_eq!(s, [i64::MIN, i64::MAX]);
        (u, s)
    }

    fn list_minmax_float(u: Vec<f32>, s: Vec<f64>) -> (Vec<f32>, Vec<f64>) {
        assert_eq!(u, [f32::MIN, f32::MAX, f32::NEG_INFINITY, f32::INFINITY]);
        assert_eq!(s, [f64::MIN, f64::MAX, f64::NEG_INFINITY, f64::INFINITY]);
        (u, s)
    }
}

pub fn main() {
    let bytes = allocated_bytes();
    test_imports();
    use test::lists::test::*;
    empty_list_param(&[]);
    empty_string_param("");
    assert!(empty_list_result().is_empty());
    assert_eq!(empty_string_result(), "");
    list_param(&[1, 2, 3, 4]);
    list_param2("foo");
    list_param3(&["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]);
    list_param4(&[
        vec!["foo".to_owned(), "bar".to_owned()],
        vec!["baz".to_owned()],
    ]);
    assert_eq!(list_result(), [1, 2, 3, 4, 5]);
    assert_eq!(list_result2(), "hello!");
    assert_eq!(list_result3(), ["hello,", "world!"]);
    assert_eq!(string_roundtrip("x"), "x");
    assert_eq!(string_roundtrip(""), "");
    assert_eq!(string_roundtrip("hello ⚑ world"), "hello ⚑ world");
    // Ensure that we properly called `free` everywhere in all the glue that we
    // needed to.
    assert_eq!(bytes, allocated_bytes());
    {
        #[link(name = "lists")]
        extern "C" {
            fn test_imports();
        }
        let _ = || {
            unsafe { test_imports() };
        };
    }
}
