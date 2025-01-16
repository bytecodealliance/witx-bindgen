wit_bindgen::generate!({
    path: "../../tests/runtime/strings",
});

struct Exports;

export!(Exports);

impl Guest for Exports {
    fn test_imports() -> () {
        test::strings::imports::take_basic("latin utf16");

        let str2 = test::strings::imports::return_unicode();
        assert_eq!(str2, "🚀🚀🚀 𠈄𓀀");
    }

    fn return_empty() -> String {
        Default::default()
    }

    fn roundtrip(s: String) -> String {
        assert!(!s.is_empty());
        s
    }
}
