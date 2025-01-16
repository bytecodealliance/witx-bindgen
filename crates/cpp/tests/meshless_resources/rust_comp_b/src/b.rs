#[allow(dead_code)]
pub mod exports {
    #[allow(dead_code)]
    pub mod foo {
        #[allow(dead_code)]
        pub mod foo {
            #[allow(dead_code, clippy::all)]
            pub mod resources {
                #[used]
                #[doc(hidden)]
                #[cfg(target_arch = "wasm32")]
                static __FORCE_SECTION_REF: fn() =
                    super::super::super::super::__link_custom_section_describing_imports;
                use super::super::super::super::_rt;
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct R {
                    handle: _rt::Resource<R>,
                }
                type _RRep<T> = Option<T>;
                impl R {
                    /// Creates a new resource from the specified representation.
                    ///
                    /// This function will create a new resource handle by moving `val` onto
                    /// the heap and then passing that heap pointer to the component model to
                    /// create a handle. The owned handle is then returned as `R`.
                    pub fn new<T: GuestR>(val: T) -> Self {
                        Self::type_guard::<T>();
                        let val: _RRep<T> = Some(val);
                        let ptr: *mut _RRep<T> = _rt::Box::into_raw(_rt::Box::new(val));
                        unsafe { Self::from_handle(T::_resource_new(ptr.cast())) }
                    }
                    /// Gets access to the underlying `T` which represents this resource.
                    pub fn get<T: GuestR>(&self) -> &T {
                        let ptr = unsafe { &*self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    /// Gets mutable access to the underlying `T` which represents this
                    /// resource.
                    pub fn get_mut<T: GuestR>(&mut self) -> &mut T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_mut().unwrap()
                    }
                    /// Consumes this resource and returns the underlying `T`.
                    pub fn into_inner<T: GuestR>(self) -> T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.take().unwrap()
                    }
                    #[doc(hidden)]
                    pub unsafe fn from_handle(handle: usize) -> Self {
                        Self {
                            handle: _rt::Resource::from_handle(handle),
                        }
                    }
                    #[doc(hidden)]
                    pub fn take_handle(&self) -> usize {
                        _rt::Resource::take_handle(&self.handle)
                    }
                    #[doc(hidden)]
                    pub fn handle(&self) -> usize {
                        _rt::Resource::handle(&self.handle)
                    }
                    #[doc(hidden)]
                    fn type_guard<T: 'static>() {
                        use core::any::TypeId;
                        static mut LAST_TYPE: Option<TypeId> = None;
                        unsafe {
                            assert!(!cfg!(target_feature = "threads"));
                            let id = TypeId::of::<T>();
                            match LAST_TYPE {
                                Some(ty) => {
                                    assert!(
                                        ty == id,
                                        "cannot use two types with this resource type"
                                    )
                                }
                                None => LAST_TYPE = Some(id),
                            }
                        }
                    }
                    #[doc(hidden)]
                    pub unsafe fn dtor<T: 'static>(handle: *mut u8) {
                        Self::type_guard::<T>();
                        let _ = _rt::Box::from_raw(handle as *mut _RRep<T>);
                    }
                    fn as_ptr<T: GuestR>(&self) -> *mut _RRep<T> {
                        R::type_guard::<T>();
                        T::_resource_rep(self.handle()).cast()
                    }
                }
                /// A borrowed version of [`R`] which represents a borrowed value
                /// with the lifetime `'a`.
                #[derive(Debug)]
                #[repr(transparent)]
                pub struct RBorrow<'a> {
                    rep: *mut u8,
                    _marker: core::marker::PhantomData<&'a R>,
                }
                impl<'a> RBorrow<'a> {
                    #[doc(hidden)]
                    pub unsafe fn lift(rep: usize) -> Self {
                        Self {
                            rep: rep as *mut u8,
                            _marker: core::marker::PhantomData,
                        }
                    }
                    /// Gets access to the underlying `T` in this resource.
                    pub fn get<T: GuestR>(&self) -> &T {
                        let ptr = unsafe { &mut *self.as_ptr::<T>() };
                        ptr.as_ref().unwrap()
                    }
                    fn as_ptr<T: 'static>(&self) -> *mut _RRep<T> {
                        R::type_guard::<T>();
                        self.rep.cast()
                    }
                }
                unsafe impl _rt::WasmResource for R {
                    #[inline]
                    unsafe fn drop(_handle: usize) {
                        {
                            #[link(wasm_import_module = "foo:foo/resources")]
                            extern "C" {
                                #[cfg_attr(target_arch = "wasm32", link_name = "[resource-drop]r")]
                                fn fooX3AfooX2FresourcesX00X5Bresource_dropX5Dr(_: usize);
                            }
                            fooX3AfooX2FresourcesX00X5Bresource_dropX5Dr(_handle);
                        }
                    }
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_constructor_r_cabi<T: GuestR>(arg0: i32) -> *mut u8 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = R::new(T::new(arg0 as u32));
                    (result0).take_handle() as *mut u8
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_method_r_add_cabi<T: GuestR>(arg0: *mut u8, arg1: i32) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::add(RBorrow::lift(arg0 as usize).get(), arg1 as u32);
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_create_cabi<T: Guest>() -> *mut u8 {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    let result0 = T::create();
                    (result0).take_handle() as *mut u8
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_consume_cabi<T: Guest>(arg0: *mut u8) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    T::consume(R::from_handle(arg0 as usize));
                }
                pub trait Guest {
                    type R: GuestR;
                    fn create() -> R;
                    /// borrows: func(o: borrow<r>);
                    fn consume(o: R) -> ();
                }
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub unsafe fn _export_drop_r_cabi<T: GuestR>(arg0: usize) {
                    #[cfg(target_arch = "wasm32")]
                    _rt::run_ctors_once();
                    R::dtor::<T>(arg0 as *mut u8);
                }
                pub trait GuestR: 'static {
                    #[doc(hidden)]
                    unsafe fn _resource_new(val: *mut u8) -> usize
                    where
                        Self: Sized,
                    {
                        val as usize
                    }
                    #[doc(hidden)]
                    fn _resource_rep(handle: usize) -> *mut u8
                    where
                        Self: Sized,
                    {
                        handle as *mut u8
                    }
                    fn new(a: u32) -> Self;
                    fn add(&self, b: u32) -> ();
                }
                #[doc(hidden)]
                macro_rules! __export_foo_foo_resources_cabi {
                    ($ty:ident with_types_in $($path_to_types:tt)*) => {
                        const _ : () = { #[cfg_attr(target_arch = "wasm32", export_name =
                        "[constructor]r")] #[cfg_attr(not(target_arch = "wasm32"),
                        no_mangle)] unsafe extern "C" fn
                        fooX3AfooX2FresourcesX00X5BconstructorX5Dr(arg0 : i32,) -> * mut
                        u8 { $($path_to_types)*:: _export_constructor_r_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::R > (arg0) } #[cfg_attr(target_arch
                        = "wasm32", export_name = "[method]r.add")]
                        #[cfg_attr(not(target_arch = "wasm32"), no_mangle)] unsafe extern
                        "C" fn fooX3AfooX2FresourcesX00X5BmethodX5DrX2Eadd(arg0 : * mut
                        u8, arg1 : i32,) { $($path_to_types)*::
                        _export_method_r_add_cabi::<<$ty as $($path_to_types)*:: Guest
                        >::R > (arg0, arg1) } #[cfg_attr(target_arch = "wasm32",
                        export_name = "create")] #[cfg_attr(not(target_arch = "wasm32"),
                        no_mangle)] unsafe extern "C" fn fooX3AfooX2FresourcesX00create()
                        -> * mut u8 { $($path_to_types)*:: _export_create_cabi::<$ty > ()
                        } #[cfg_attr(target_arch = "wasm32", export_name = "consume")]
                        #[cfg_attr(not(target_arch = "wasm32"), no_mangle)] unsafe extern
                        "C" fn fooX3AfooX2FresourcesX00consume(arg0 : * mut u8,) {
                        $($path_to_types)*:: _export_consume_cabi::<$ty > (arg0) }
                        #[cfg_attr(not(target_arch = "wasm32"), no_mangle)] unsafe extern
                        "C" fn fooX3AfooX2FresourcesX00X5Bresource_dropX5Dr(arg0 : usize)
                        { $($path_to_types)*:: _export_drop_r_cabi::<<$ty as
                        $($path_to_types)*:: Guest >::R > (arg0) } };
                    };
                }
                #[doc(hidden)]
                pub(crate) use __export_foo_foo_resources_cabi;
            }
        }
    }
}
mod _rt {
    use core::fmt;
    use core::marker;
    use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
    /// A type which represents a component model resource, either imported or
    /// exported into this component.
    ///
    /// This is a low-level wrapper which handles the lifetime of the resource
    /// (namely this has a destructor). The `T` provided defines the component model
    /// intrinsics that this wrapper uses.
    ///
    /// One of the chief purposes of this type is to provide `Deref` implementations
    /// to access the underlying data when it is owned.
    ///
    /// This type is primarily used in generated code for exported and imported
    /// resources.
    #[repr(transparent)]
    pub struct Resource<T: WasmResource> {
        handle: AtomicUsize,
        _marker: marker::PhantomData<T>,
    }
    /// A trait which all wasm resources implement, namely providing the ability to
    /// drop a resource.
    ///
    /// This generally is implemented by generated code, not user-facing code.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait WasmResource {
        /// Invokes the `[resource-drop]...` intrinsic.
        unsafe fn drop(handle: usize);
    }
    impl<T: WasmResource> Resource<T> {
        #[doc(hidden)]
        pub unsafe fn from_handle(handle: usize) -> Self {
            debug_assert!(handle != 0);
            Self {
                handle: AtomicUsize::new(handle),
                _marker: marker::PhantomData,
            }
        }
        /// Takes ownership of the handle owned by `resource`.
        ///
        /// Note that this ideally would be `into_handle` taking `Resource<T>` by
        /// ownership. The code generator does not enable that in all situations,
        /// unfortunately, so this is provided instead.
        ///
        /// Also note that `take_handle` is in theory only ever called on values
        /// owned by a generated function. For example a generated function might
        /// take `Resource<T>` as an argument but then call `take_handle` on a
        /// reference to that argument. In that sense the dynamic nature of
        /// `take_handle` should only be exposed internally to generated code, not
        /// to user code.
        #[doc(hidden)]
        pub fn take_handle(resource: &Resource<T>) -> usize {
            resource.handle.swap(0, Relaxed)
        }
        #[doc(hidden)]
        pub fn handle(resource: &Resource<T>) -> usize {
            resource.handle.load(Relaxed)
        }
    }
    impl<T: WasmResource> fmt::Debug for Resource<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Resource")
                .field("handle", &self.handle)
                .finish()
        }
    }
    impl<T: WasmResource> Drop for Resource<T> {
        fn drop(&mut self) {
            unsafe {
                match self.handle.load(Relaxed) {
                    0 => {}
                    other => T::drop(other),
                }
            }
        }
    }
    pub use alloc_crate::boxed::Box;
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
macro_rules! __export_b_impl {
    ($ty:ident) => {
        self::export!($ty with_types_in self);
    };
    ($ty:ident with_types_in $($path_to_types_root:tt)*) => {
        $($path_to_types_root)*::
        exports::foo::foo::resources::__export_foo_foo_resources_cabi!($ty with_types_in
        $($path_to_types_root)*:: exports::foo::foo::resources);
    };
}
#[doc(inline)]
pub(crate) use __export_b_impl as export;
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.28.0:b:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 272] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x98\x01\x01A\x02\x01\
A\x02\x01B\x0b\x04\0\x01r\x03\x01\x01i\0\x01@\x01\x01ay\0\x01\x04\0\x0e[construc\
tor]r\x01\x02\x01h\0\x01@\x02\x04self\x03\x01by\x01\0\x04\0\x0d[method]r.add\x01\
\x04\x01@\0\0\x01\x04\0\x06create\x01\x05\x01@\x01\x01o\x01\x01\0\x04\0\x07consu\
me\x01\x06\x04\x01\x11foo:foo/resources\x05\0\x04\x01\x09foo:foo/b\x04\0\x0b\x07\
\x01\0\x01b\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x07\
0.214.0\x10wit-bindgen-rust\x060.28.0";
#[inline(never)]
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen::rt::maybe_link_cabi_realloc();
}
