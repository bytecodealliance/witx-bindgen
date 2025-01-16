wit_bindgen::generate!({
    path: "../tests/runtime/records",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

pub struct MyExports;

use exports::test::records::test as test_imports;

impl test_imports::Guest for MyExports {
    fn multiple_results() -> (u8, u16) {
        (4, 5)
    }

    fn swap_tuple(a: (u8, u32)) -> (u32, u8) {
        (a.1, a.0)
    }

    fn roundtrip_flags1(a: test_imports::F1) -> test_imports::F1 {
        drop(format!("{:?}", a));
        let _ = a & test_imports::F1::all();
        a
    }

    fn roundtrip_flags2(a: test_imports::F2) -> test_imports::F2 {
        a
    }

    fn roundtrip_flags3(
        a: test_imports::Flag8,
        b: test_imports::Flag16,
        c: test_imports::Flag32,
    ) -> (
        test_imports::Flag8,
        test_imports::Flag16,
        test_imports::Flag32,
    ) {
        (a, b, c)
    }

    fn roundtrip_record1(a: test_imports::R1) -> test_imports::R1 {
        drop(format!("{:?}", a));
        a
    }

    fn tuple1(a: (u8,)) -> (u8,) {
        (a.0,)
    }
}

pub fn main() {
    use test::records::test::*;

    test_imports();
    assert_eq!(multiple_results(), (100, 200));
    assert_eq!(swap_tuple((1u8, 2u32)), (2u32, 1u8));
    assert_eq!(roundtrip_flags1(F1::A), F1::A);
    assert_eq!(roundtrip_flags1(F1::empty()), F1::empty());
    assert_eq!(roundtrip_flags1(F1::B), F1::B);
    assert_eq!(roundtrip_flags1(F1::A | F1::B), F1::A | F1::B);

    assert_eq!(roundtrip_flags2(F2::C), F2::C);
    assert_eq!(roundtrip_flags2(F2::empty()), F2::empty());
    assert_eq!(roundtrip_flags2(F2::D), F2::D);
    assert_eq!(roundtrip_flags2(F2::C | F2::E), F2::C | F2::E);

    let r = roundtrip_record1(R1 {
        a: 8,
        b: F1::empty(),
    });
    assert_eq!(r.a, 8);
    assert_eq!(r.b, F1::empty());

    let r = roundtrip_record1(R1 {
        a: 0,
        b: F1::A | F1::B,
    });
    assert_eq!(r.a, 0);
    assert_eq!(r.b, F1::A | F1::B);

    assert_eq!(tuple1((1,)), (1,));
    {
        #[link(name = "records")]
        extern "C" {
            fn test_imports();
        }
        let _ = || {
            unsafe { test_imports() };
        };
    }
}
