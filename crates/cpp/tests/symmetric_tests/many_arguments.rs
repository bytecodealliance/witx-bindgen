wit_bindgen::generate!({
    path: "../tests/runtime/many_arguments",
    symmetric: true,
    invert_direction: true,
});

export!(MyExports);

pub struct MyExports {}

impl exports::imports::Guest for MyExports {
    fn many_arguments(
        a1: u64,
        a2: u64,
        a3: u64,
        a4: u64,
        a5: u64,
        a6: u64,
        a7: u64,
        a8: u64,
        a9: u64,
        a10: u64,
        a11: u64,
        a12: u64,
        a13: u64,
        a14: u64,
        a15: u64,
        a16: u64,
    ) {
        assert_eq!(a1, 1);
        assert_eq!(a2, 2);
        assert_eq!(a3, 3);
        assert_eq!(a4, 4);
        assert_eq!(a5, 5);
        assert_eq!(a6, 6);
        assert_eq!(a7, 7);
        assert_eq!(a8, 8);
        assert_eq!(a9, 9);
        assert_eq!(a10, 10);
        assert_eq!(a11, 11);
        assert_eq!(a12, 12);
        assert_eq!(a13, 13);
        assert_eq!(a14, 14);
        assert_eq!(a15, 15);
        assert_eq!(a16, 16);
    }
}

fn main() {
    many_arguments(
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        9,
        10,
        11,
        12,
        13,
        14,
        15,
        16,
    );
    {
        #[link(name = "many_arguments")]
        extern "C" {
            fn many_arguments(a1: i64,
                a2: i64,
                a3: i64,
                a4: i64,
                a5: i64,
                a6: i64,
                a7: i64,
                a8: i64,
                a9: i64,
                a10: i64,
                a11: i64,
                a12: i64,
                a13: i64,
                a14: i64,
                a15: i64,
                a16: i64,);
        }
        let _ = || {
            unsafe { many_arguments(0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,) };
        };
    }
}
