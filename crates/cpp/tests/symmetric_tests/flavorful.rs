wit_bindgen::generate!({
    path: "../tests/runtime/flavorful",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

use std::sync::atomic::AtomicBool;

use exports::test::flavorful::test as test_imports;
use test::flavorful::test::*;

#[derive(Default)]
pub struct MyExports;

static ERRORED: AtomicBool = AtomicBool::new(false);

impl exports::test::flavorful::test::Guest for MyExports {
    fn f_list_in_record1(ty: test_imports::ListInRecord1) {
        assert_eq!(ty.a, "list_in_record1");
    }

    fn f_list_in_record2() -> test_imports::ListInRecord2 {
        test_imports::ListInRecord2 {
            a: "list_in_record2".to_string(),
        }
    }

    fn f_list_in_record3(a: test_imports::ListInRecord3) -> test_imports::ListInRecord3 {
        assert_eq!(a.a, "list_in_record3 input");
        test_imports::ListInRecord3 {
            a: "list_in_record3 output".to_string(),
        }
    }

    fn f_list_in_record4(a: test_imports::ListInAlias) -> test_imports::ListInAlias {
        assert_eq!(a.a, "input4");
        test_imports::ListInRecord4 {
            a: "result4".to_string(),
        }
    }

    fn f_list_in_variant1(a: test_imports::ListInVariant1V1, b: test_imports::ListInVariant1V2) {
        assert_eq!(a.unwrap(), "foo");
        assert_eq!(b.unwrap_err(), "bar");
    }

    fn f_list_in_variant2() -> Option<String> {
        Some("list_in_variant2".to_string())
    }

    fn f_list_in_variant3(a: test_imports::ListInVariant3) -> Option<String> {
        assert_eq!(a.unwrap(), "input3");
        Some("output3".to_string())
    }

    fn errno_result() -> Result<(), test_imports::MyErrno> {
        if ERRORED.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        test_imports::MyErrno::A.to_string();
        format!("{:?}", test_imports::MyErrno::A);
        fn assert_error<T: std::error::Error>() {}
        assert_error::<test_imports::MyErrno>();
        ERRORED.store(true, std::sync::atomic::Ordering::SeqCst);
        Err(test_imports::MyErrno::B)
    }

    fn list_typedefs(
        a: test_imports::ListTypedef,
        b: test_imports::ListTypedef3,
    ) -> (test_imports::ListTypedef2, test_imports::ListTypedef3) {
        assert_eq!(a, "typedef1");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], "typedef2");
        (b"typedef3".to_vec(), vec!["typedef4".to_string()])
    }

    fn list_of_variants(
        bools: Vec<bool>,
        results: Vec<Result<(), ()>>,
        enums: Vec<test_imports::MyErrno>,
    ) -> (Vec<bool>, Vec<Result<(), ()>>, Vec<test_imports::MyErrno>) {
        assert_eq!(bools, [true, false]);
        assert_eq!(results, [Ok(()), Err(())]);
        assert_eq!(
            enums,
            [test_imports::MyErrno::Success, test_imports::MyErrno::A]
        );
        (
            vec![false, true],
            vec![Err(()), Ok(())],
            vec![test_imports::MyErrno::A, test_imports::MyErrno::B],
        )
    }
}

pub fn main() {
    test_imports();
    // let exports = exports.test_flavorful_test();

    f_list_in_record1(&ListInRecord1 {
        a: "list_in_record1".to_string(),
    });
    assert_eq!(f_list_in_record2().a, "list_in_record2");

    assert_eq!(
        f_list_in_record3(&ListInRecord3 {
            a: "list_in_record3 input".to_string()
        })
        .a,
        "list_in_record3 output"
    );

    assert_eq!(
        f_list_in_record4(&ListInAlias {
            a: "input4".to_string()
        })
        .a,
        "result4"
    );

    f_list_in_variant1(&Some("foo".to_string()), &Err("bar".to_string()));
    assert_eq!(f_list_in_variant2(), Some("list_in_variant2".to_string()));
    assert_eq!(
        f_list_in_variant3(&Some("input3".to_string())),
        Some("output3".to_string())
    );

    assert!(errno_result().is_err());
    MyErrno::A.to_string();
    format!("{:?}", MyErrno::A);
    fn assert_error<T: std::error::Error>() {}
    assert_error::<MyErrno>();

    let (a, b) = list_typedefs(&"typedef1".to_string(), &vec!["typedef2".to_string()]);
    assert_eq!(a, b"typedef3");
    assert_eq!(b.len(), 1);
    assert_eq!(b[0], "typedef4");
    {
        #[link(name = "flavorful")]
        extern "C" {
            fn test_imports();
        }
        let _ = || {
            unsafe { test_imports() };
        };
    }
}
