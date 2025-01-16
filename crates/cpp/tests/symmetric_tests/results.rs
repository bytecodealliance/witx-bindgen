wit_bindgen::generate!({
    path: "../tests/runtime/results",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

pub struct MyExports;

use exports::test::results::test as imports;

impl exports::test::results::test::Guest for MyExports {
    fn string_error(a: f32) -> Result<f32, String> {
        if a == 0.0 {
            Err("zero".to_owned())
        } else {
            Ok(a)
        }
    }

    fn enum_error(a: f32) -> Result<f32, imports::E> {
        if a == 0.0 {
            Err(imports::E::A)
        } else {
            Ok(a)
        }
    }

    fn record_error(a: f32) -> Result<f32, imports::E2> {
        if a == 0.0 {
            Err(imports::E2 {
                line: 420,
                column: 0,
            })
        } else if a == 1.0 {
            Err(imports::E2 {
                line: 77,
                column: 2,
            })
        } else {
            Ok(a)
        }
    }

    fn variant_error(a: f32) -> Result<f32, imports::E3> {
        if a == 0.0 {
            Err(imports::E3::E2(imports::E2 {
                line: 420,
                column: 0,
            }))
        } else if a == 1.0 {
            Err(imports::E3::E1(imports::E::B))
        } else if a == 2.0 {
            Err(imports::E3::E1(imports::E::C))
        } else {
            Ok(a)
        }
    }

    fn empty_error(a: u32) -> Result<u32, ()> {
        if a == 0 {
            Err(())
        } else if a == 1 {
            Ok(42)
        } else {
            Ok(a)
        }
    }

    fn double_error(a: u32) -> Result<Result<(), String>, String> {
        if a == 0 {
            Ok(Ok(()))
        } else if a == 1 {
            Ok(Err("one".into()))
        } else {
            Err("two".into())
        }
    }
}

pub fn main() {
    use test::results::test::{
        double_error, empty_error, enum_error, record_error, string_error, variant_error, E, E2, E3,
    };

    assert_eq!(string_error(0.0), Err("zero".to_owned()));
    assert_eq!(string_error(1.0), Ok(1.0));

    assert_eq!(enum_error(0.0), Err(E::A));
    assert_eq!(enum_error(0.0), Err(E::A));

    assert!(matches!(
        record_error(0.0),
        Err(E2 {
            line: 420,
            column: 0
        })
    ));
    assert!(matches!(
        record_error(1.0),
        Err(E2 {
            line: 77,
            column: 2
        })
    ));

    assert!(record_error(2.0).is_ok());

    assert!(matches!(
        variant_error(0.0),
        Err(E3::E2(E2 {
            line: 420,
            column: 0
        }))
    ));
    assert!(matches!(variant_error(1.0), Err(E3::E1(E::B))));
    assert!(matches!(variant_error(2.0), Err(E3::E1(E::C))));

    assert_eq!(empty_error(0), Err(()));
    assert_eq!(empty_error(1), Ok(42));
    assert_eq!(empty_error(2), Ok(2));

    assert_eq!(double_error(0), Ok(Ok(())));
    assert_eq!(double_error(1), Ok(Err("one".into())));
    assert_eq!(double_error(2), Err("two".into()));
    {
        #[link(name = "results")]
        extern "C" {
            fn exp_testX3AresultsX2FtestX00string_error(a: f32, b: *mut u8);
        }
        let _ = || {
            unsafe { exp_testX3AresultsX2FtestX00string_error(0.0, std::ptr::null_mut()) };
        };
    }
}
