wit_bindgen::generate!({
    path: "../tests/runtime/strings",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

pub struct MyExports;

impl exports::test::strings::imports::Guest for MyExports {
    fn take_basic(s: String) {
        assert_eq!(s, "latin utf16");
    }

    fn return_unicode() -> String {
        "🚀🚀🚀 𠈄𓀀".to_string()
    }
}

pub fn main() {
    test_imports();
    assert_eq!(return_empty(), "");
    assert_eq!(roundtrip("str"), "str");
    assert_eq!(
        roundtrip("🚀🚀🚀 𠈄𓀀"),
        "🚀🚀🚀 𠈄𓀀"
    );
    {
        #[link(name = "strings")]
        extern "C" {
            fn roundtrip(_: *mut u8, _: usize, _: *mut u8);
        }
        let _ = || {
            unsafe { roundtrip(core::ptr::null_mut(), 0, core::ptr::null_mut()) };
        };
    }
}
