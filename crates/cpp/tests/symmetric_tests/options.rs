wit_bindgen::generate!({
    path: "../tests/runtime/options",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

pub struct MyExports;

impl exports::test::options::test::Guest for MyExports {
    fn option_none_param(a: Option<String>) {
        assert!(a.is_none());
    }

    fn option_none_result() -> Option<String> {
        None
    }

    fn option_some_param(a: Option<String>) {
        assert_eq!(a, Some("foo".to_string()));
    }

    fn option_some_result() -> Option<String> {
        Some("foo".to_string())
    }

    fn option_roundtrip(a: Option<String>) -> Option<String> {
        a
    }

    fn double_option_roundtrip(a: Option<Option<u32>>) -> Option<Option<u32>> {
        a
    }
}

pub fn main() {
    use test::options::test::*;
    test_imports();
    assert!(option_none_result().is_none());
    assert_eq!(option_some_result(), Some("foo".to_string()));
    option_none_param(None);
    option_some_param(Some("foo"));
    assert_eq!(option_roundtrip(Some("foo")), Some("foo".to_string()));
    assert_eq!(double_option_roundtrip(Some(Some(42))), Some(Some(42)));
    assert_eq!(double_option_roundtrip(Some(None)), Some(None));
    assert_eq!(double_option_roundtrip(None), None);
    {
        #[link(name = "options")]
        extern "C" {
            fn test_imports();
        }
        let _ = || {
            unsafe { test_imports() };
        };
    }
}
