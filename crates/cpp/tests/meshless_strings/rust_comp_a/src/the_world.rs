#[allow(dead_code)]
pub mod foo {
    #[allow(dead_code)]
    pub mod foo {
        #[allow(dead_code, clippy::all)]
        pub mod strings {
            #[used]
            #[doc(hidden)]
            #[cfg(target_arch = "wasm32")]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            #[allow(unused_unsafe, clippy::all)]
            pub fn a(x: &str) -> () {
                unsafe {
                    let vec0 = x;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    #[link(wasm_import_module = "foo:foo/strings")]
                    extern "C" {
                        #[cfg_attr(target_arch = "wasm32", link_name = "a")]
                        fn fooX3AfooX2FstringsX00a(_: *mut u8, _: usize);
                    }
                    fooX3AfooX2FstringsX00a(ptr0.cast_mut(), len0);
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn b() -> _rt::String {
                unsafe {
                    #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
                    #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
                    struct RetArea(
                        [::core::mem::MaybeUninit<
                            u8,
                        >; (2 * core::mem::size_of::<*const u8>())],
                    );
                    let mut ret_area = RetArea(
                        [::core::mem::MaybeUninit::uninit(); (2
                            * core::mem::size_of::<*const u8>())],
                    );
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[link(wasm_import_module = "foo:foo/strings")]
                    extern "C" {
                        #[cfg_attr(target_arch = "wasm32", link_name = "b")]
                        fn fooX3AfooX2FstringsX00b(_: *mut u8);
                    }
                    fooX3AfooX2FstringsX00b(ptr0);
                    let l1 = *ptr0.add(0).cast::<*mut u8>();
                    let l2 = *ptr0
                        .add(core::mem::size_of::<*const u8>())
                        .cast::<usize>();
                    let len3 = l2;
                    let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                    _rt::string_lift(bytes3)
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            pub fn c(a: &str, b: &str) -> _rt::String {
                unsafe {
                    #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
                    #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
                    struct RetArea(
                        [::core::mem::MaybeUninit<
                            u8,
                        >; (2 * core::mem::size_of::<*const u8>())],
                    );
                    let mut ret_area = RetArea(
                        [::core::mem::MaybeUninit::uninit(); (2
                            * core::mem::size_of::<*const u8>())],
                    );
                    let vec0 = a;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    let vec1 = b;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let ptr2 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[link(wasm_import_module = "foo:foo/strings")]
                    extern "C" {
                        #[cfg_attr(target_arch = "wasm32", link_name = "c")]
                        fn fooX3AfooX2FstringsX00c(
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                        );
                    }
                    fooX3AfooX2FstringsX00c(
                        ptr0.cast_mut(),
                        len0,
                        ptr1.cast_mut(),
                        len1,
                        ptr2,
                    );
                    let l3 = *ptr2.add(0).cast::<*mut u8>();
                    let l4 = *ptr2
                        .add(core::mem::size_of::<*const u8>())
                        .cast::<usize>();
                    let len5 = l4;
                    let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                    _rt::string_lift(bytes5)
                }
            }
        }
    }
}
#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod foo {
        #[allow(dead_code)]
        pub mod foo {
            #[allow(dead_code, clippy::all)]
            pub mod strings {
                #[used]
                #[doc(hidden)]
                #[cfg(target_arch = "wasm32")]
                static __FORCE_SECTION_REF: fn() = super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_a_cabi<T: Guest>(arg0: *mut u8, arg1: usize) {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let string0 = String::from(
                        std::str::from_utf8(std::slice::from_raw_parts(arg0, len0))
                            .unwrap(),
                    );
                    T::a(string0);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_b_cabi<T: Guest>(arg0: *mut u8) {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let result0 = T::b();
                    let vec1 = (result0.into_bytes()).into_boxed_slice();
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    ::core::mem::forget(vec1);
                    *arg0.add(core::mem::size_of::<*const u8>()).cast::<usize>() = len1;
                    *arg0.add(0).cast::<*mut u8>() = ptr1.cast_mut();
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_c_cabi<T: Guest>(
                    arg0: *mut u8,
                    arg1: usize,
                    arg2: *mut u8,
                    arg3: usize,
                    arg4: *mut u8,
                ) {
                    #[cfg(target_arch = "wasm32")] _rt::run_ctors_once();
                    let len0 = arg1;
                    let string0 = String::from(
                        std::str::from_utf8(std::slice::from_raw_parts(arg0, len0))
                            .unwrap(),
                    );
                    let len1 = arg3;
                    let string1 = String::from(
                        std::str::from_utf8(std::slice::from_raw_parts(arg2, len1))
                            .unwrap(),
                    );
                    let result2 = T::c(string0, string1);
                    let vec3 = (result2.into_bytes()).into_boxed_slice();
                    let ptr3 = vec3.as_ptr().cast::<u8>();
                    let len3 = vec3.len();
                    ::core::mem::forget(vec3);
                    *arg4.add(core::mem::size_of::<*const u8>()).cast::<usize>() = len3;
                    *arg4.add(0).cast::<*mut u8>() = ptr3.cast_mut();
                }
                pub trait Guest {
                    fn a(x: _rt::String) -> ();
                    fn b() -> _rt::String;
                    fn c(a: _rt::String, b: _rt::String) -> _rt::String;
                }
                #[doc(hidden)]
                macro_rules! __export_foo_foo_strings_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[cfg_attr(target_arch = "wasm32", export_name =
                        "a")] #[cfg_attr(not(target_arch = "wasm32"), no_mangle)] unsafe
                        extern "C" fn a_fooX3AfooX2FstringsX00a(arg0 : * mut u8, arg1 :
                        usize,) { $($path_to_types)*:: _export_a_cabi::<$ty > (arg0,
                        arg1) } #[cfg_attr(target_arch = "wasm32", export_name = "b")]
                        #[cfg_attr(not(target_arch = "wasm32"), no_mangle)] unsafe extern
                        "C" fn a_fooX3AfooX2FstringsX00b(arg0 : * mut u8,) {
                        $($path_to_types)*:: _export_b_cabi::<$ty > (arg0) }
                        #[cfg_attr(target_arch = "wasm32", export_name = "c")]
                        #[cfg_attr(not(target_arch = "wasm32"), no_mangle)] unsafe extern
                        "C" fn a_fooX3AfooX2FstringsX00c(arg0 : * mut u8, arg1 : usize,
                        arg2 : * mut u8, arg3 : usize, arg4 : * mut u8,) {
                        $($path_to_types)*:: _export_c_cabi::<$ty > (arg0, arg1, arg2,
                        arg3, arg4) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_foo_foo_strings_cabi;
            }
        }
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen::rt::run_ctors_once();
    }
    extern crate alloc as alloc_crate;
}
/// Generates `#[no_mangle]` functions to export the specified type as the
/// root implementation of all generated traits.
///
/// For more information see the documentation of `wit_bindgen::generate!`.
///
/// ```rust
/// # macro_rules! export{ ($($t:tt)*) => (); }
/// # trait Guest {}
/// struct MyType;
///
/// impl Guest for MyType {
///     // ...
/// }
///
/// export!(MyType);
/// ```
#[allow(unused_macros)]
#[doc(hidden)]
macro_rules! __export_the_world_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::foo::foo::strings::__export_foo_foo_strings_cabi!($ty with_types_in
        $($path_to_types_root)*:: exports::foo::foo::strings);
    };
}
#[doc(inline)]
pub(crate) use __export_the_world_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.28.0:the-world:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 286] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x9e\x01\x01A\x02\x01\
A\x04\x01B\x06\x01@\x01\x01xs\x01\0\x04\0\x01a\x01\0\x01@\0\0s\x04\0\x01b\x01\x01\
\x01@\x02\x01as\x01bs\0s\x04\0\x01c\x01\x02\x03\x01\x0ffoo:foo/strings\x05\0\x01\
B\x06\x01@\x01\x01xs\x01\0\x04\0\x01a\x01\0\x01@\0\0s\x04\0\x01b\x01\x01\x01@\x02\
\x01as\x01bs\0s\x04\0\x01c\x01\x02\x04\x01\x0ffoo:foo/strings\x05\x01\x04\x01\x11\
foo:foo/the-world\x04\0\x0b\x0f\x01\0\x09the-world\x03\0\0\0G\x09producers\x01\x0c\
processed-by\x02\x0dwit-component\x070.215.0\x10wit-bindgen-rust\x060.28.0";
#[inline(never)]
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen::rt::maybe_link_cabi_realloc();
}
