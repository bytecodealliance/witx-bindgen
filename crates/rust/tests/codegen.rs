#![allow(unused_macros)]

mod codegen_tests {
    macro_rules! codegen_test {
        ($id:ident $name:tt $test:tt) => {
            mod $id {
                wit_bindgen::generate!(in $test);

                // This empty module named 'core' is here to catch module path
                // conflicts with 'core' modules used in code generated by the
                // wit_bindgen::generate macro.
                // Ref: https://github.com/bytecodealliance/wit-bindgen/pull/568
                mod core {}

                #[test]
                fn works() {}

                mod duplicate {
                    wit_bindgen::generate!({
                        path: $test,
                        ownership: Borrowing {
                            duplicate_if_necessary: true
                        }
                    });

                    #[test]
                    fn works() {}
                }
            }

        };
    }
    test_helpers::codegen_tests!();
}

mod strings {
    wit_bindgen::generate!({
        inline: "
            package my:strings

            world not-used-name {
                import cat: interface {
                    foo: func(x: string)
                    bar: func() -> string
                }
            }
        ",
    });

    #[allow(dead_code)]
    fn test() {
        // Test the argument is `&str`.
        cat::foo("hello");

        // Test the return type is `String`.
        let _t: String = cat::bar();
    }
}

/// Like `strings` but with raw_strings`.
mod raw_strings {
    wit_bindgen::generate!({
        inline: "
            package my:raw-strings

            world not-used-name {
                import cat: interface {
                    foo: func(x: string)
                    bar: func() -> string
                }
            }
        ",
        raw_strings,
    });

    #[allow(dead_code)]
    fn test() {
        // Test the argument is `&[u8]`.
        cat::foo(b"hello");

        // Test the return type is `Vec<u8>`.
        let _t: Vec<u8> = cat::bar();
    }
}

// This is a static compilation test to ensure that
// export bindings can go inside of another mod/crate
// and still compile.
mod prefix {
    mod bindings {
        wit_bindgen::generate!({
            inline: "
                package my:prefix

                world baz {
                    export exports1: interface {
                        foo: func(x: string)
                        bar: func() -> string
                    }
                }
            ",
            macro_call_prefix: "bindings::"
        });

        pub(crate) use export_baz;
    }

    struct Component;

    impl bindings::exports::exports1::Exports1 for Component {
        fn foo(x: String) {
            println!("foo: {}", x);
        }

        fn bar() -> String {
            "bar".to_string()
        }
    }

    bindings::export_baz!(Component);
}

// This is a static compilation test to check that
// the export macro name can be overridden.
mod macro_name {
    wit_bindgen::generate!({
        inline: "
            package my:macro-name

            world baz {
                export exports2: interface {
                    foo: func(x: string)
                }
            }
        ",
        export_macro_name: "jam"
    });

    struct Component;

    impl exports::exports2::Exports2 for Component {
        fn foo(x: String) {
            println!("foo: {}", x);
        }
    }

    jam!(Component);
}

mod skip {
    wit_bindgen::generate!({
        inline: "
            package my:inline

            world baz {
                export exports: interface {
                    foo: func()
                    bar: func()
                }
            }
        ",
        skip: ["foo"],
    });

    struct Component;

    impl exports::exports::Exports for Component {
        fn bar() {}
    }

    export_baz!(Component);
}

mod symbol_does_not_conflict {
    wit_bindgen::generate!({
        inline: "
            package my:inline

            interface foo1 {
                foo: func()
            }

            interface foo2 {
                foo: func()
            }

            interface bar1 {
                bar: func() -> string
            }

            interface bar2 {
                bar: func() -> string
            }


            world foo {
                export foo1
                export foo2
                export bar1
                export bar2
            }
        ",
    });

    struct Component;

    impl exports::my::inline::foo1::Foo1 for Component {
        fn foo() {}
    }

    impl exports::my::inline::foo2::Foo2 for Component {
        fn foo() {}
    }

    impl exports::my::inline::bar1::Bar1 for Component {
        fn bar() -> String {
            String::new()
        }
    }

    impl exports::my::inline::bar2::Bar2 for Component {
        fn bar() -> String {
            String::new()
        }
    }

    export_foo!(Component);
}

mod alternative_runtime_path {
    wit_bindgen::generate!({
        inline: "
            package my:inline
            world foo {
                export foo: func()
            }
        ",
        runtime_path: "my_rt",
    });

    pub(crate) use wit_bindgen::rt as my_rt;

    struct Component;

    impl Foo for Component {
        fn foo() {}
    }

    export_foo!(Component);
}
