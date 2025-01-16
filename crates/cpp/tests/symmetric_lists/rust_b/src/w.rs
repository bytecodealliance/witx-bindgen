#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod test {
        #[allow(dead_code)]
        pub mod test {
            #[allow(dead_code, clippy::all)]
            pub mod i {
                #[used]
                #[doc(hidden)]
                static __FORCE_SECTION_REF: fn() =
                    super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_f_cabi<T: Guest>(arg0: *mut u8, arg1: usize, arg2: *mut u8) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let base3 = arg0;
                    let len3 = arg1;
                    let mut result3 = _rt::Vec::with_capacity(len3);
                    for i in 0..len3 {
                        let base = base3.add(i * (2 * core::mem::size_of::<*const u8>()));
                        let e3 = {
                            let l0 = *base.add(0).cast::<*mut u8>();
                            let l1 = *base.add(core::mem::size_of::<*const u8>()).cast::<usize>();
                            let len2 = l1;
                            let string2 = String::from(
                                std::str::from_utf8(std::slice::from_raw_parts(l0, len2)).unwrap(),
                            );
                            string2
                        };
                        result3.push(e3);
                    }
                    let result4 = T::f(result3);
                    let vec6 = result4;
                    let len6 = vec6.len();
                    let layout6 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec6.len() * (2 * core::mem::size_of::<*const u8>()),
                        core::mem::size_of::<*const u8>(),
                    );
                    let result6 = if layout6.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout6).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout6);
                        }
                        ptr
                    } else {
                        ::core::ptr::null_mut()
                    };
                    for (i, e) in vec6.into_iter().enumerate() {
                        let base = result6.add(i * (2 * core::mem::size_of::<*const u8>()));
                        {
                            let vec5 = (e.into_bytes()).into_boxed_slice();
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            ::core::mem::forget(vec5);
                            *base.add(core::mem::size_of::<*const u8>()).cast::<usize>() = len5;
                            *base.add(0).cast::<*mut u8>() = ptr5.cast_mut();
                        }
                    }
                    *arg2.add(core::mem::size_of::<*const u8>()).cast::<usize>() = len6;
                    *arg2.add(0).cast::<*mut u8>() = result6;
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_g_cabi<T: Guest>(arg0: *mut u8, arg1: usize, arg2: *mut u8) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let len0 = arg1;
                    let result1 =
                        T::g(unsafe { std::slice::from_raw_parts(arg0.cast(), len0) }.to_vec());
                    let vec2 = (result1).into_boxed_slice();
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    ::core::mem::forget(vec2);
                    *arg2.add(core::mem::size_of::<*const u8>()).cast::<usize>() = len2;
                    *arg2.add(0).cast::<*mut u8>() = ptr2.cast_mut();
                }
                pub trait Guest {
                    fn f(a: _rt::Vec<_rt::String>) -> _rt::Vec<_rt::String>;
                    fn g(a: _rt::Vec<u8>) -> _rt::Vec<u8>;
                }
                #[doc(hidden)]
                macro_rules! __export_test_test_i_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[cfg_attr(target_arch = "wasm32", export_name =
                        "f")] #[cfg_attr(not(target_arch = "wasm32"), no_mangle)] unsafe
                        extern "C" fn testX3AtestX2FiX00f(arg0 : * mut u8, arg1 : usize,
                        arg2 : * mut u8,) { $($path_to_types)*:: _export_f_cabi::<$ty >
                        (arg0, arg1, arg2) } #[cfg_attr(target_arch = "wasm32",
                        export_name = "g")] #[cfg_attr(not(target_arch = "wasm32"),
                        no_mangle)] unsafe extern "C" fn testX3AtestX2FiX00g(arg0 : * mut
                        u8, arg1 : usize, arg2 : * mut u8,) { $($path_to_types)*::
                        _export_g_cabi::<$ty > (arg0, arg1, arg2) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_test_test_i_cabi;
            }
        }
    }
}
mod _rt {
    #[cfg(target_arch = "wasm32")]
    pub fn run_ctors_once() {
        wit_bindgen::rt::run_ctors_once();
    }
    pub use alloc_crate::alloc;
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
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
macro_rules! __export_w_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*:: exports::test::test::i::__export_test_test_i_cabi!($ty
        with_types_in $($path_to_types_root)*:: exports::test::test::i);
    };
}
#[doc(inline)]
pub(crate) use __export_w_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:test:test:w:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 194] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07K\x01A\x02\x01A\x02\x01\
B\x06\x01ps\x01@\x01\x01a\0\0\0\x04\0\x01f\x01\x01\x01p}\x01@\x01\x01a\x02\0\x02\
\x04\0\x01g\x01\x03\x04\x01\x0btest:test/i\x05\0\x04\x01\x0btest:test/w\x04\0\x0b\
\x07\x01\0\x01w\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\
\x070.216.0\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen::rt::maybe_link_cabi_realloc();
}
