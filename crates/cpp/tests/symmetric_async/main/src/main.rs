use wit_bindgen_symmetric_rt::{CallbackState, EventSubscription};

#[link(name = "async_module")]
extern "C" {
    pub fn X5BasyncX5DtestX3AtestX2Fstring_delayX00forward(
        addr: *const u8,
        len: usize,
        results: *mut (),
    ) -> *mut ();
}

extern "C" fn print_result(obj: *mut ()) -> CallbackState {
    let addrptr = unsafe { *obj.cast::<*mut u8>() };
    let lenptr = unsafe { obj.byte_add(core::mem::size_of::<*const u8>()) };
    let len = unsafe { *lenptr.cast::<usize>() };
    let vec = unsafe { Vec::from_raw_parts(addrptr, len, len) };
    let string = std::str::from_utf8(&vec).unwrap();
    println!("Result {string}");
    CallbackState::Ready
}

fn main() {
    let mut result1: [usize; 2] = [0, 0];
    let handle1 = unsafe {
        X5BasyncX5DtestX3AtestX2Fstring_delayX00forward(
            "A".as_ptr(),
            1,
            result1.as_mut_ptr().cast(),
        )
    };
    assert_eq!(handle1, core::ptr::null_mut());
    let vec = unsafe { Vec::from_raw_parts(result1[0] as *mut u8, result1[1], result1[1]) };
    let string = std::str::from_utf8(&vec).unwrap();
    println!("Result {string}");

    let mut result2: [usize; 2] = [0, 0];
    let handle2 = unsafe {
        X5BasyncX5DtestX3AtestX2Fstring_delayX00forward(
            "B".as_ptr(),
            1,
            result2.as_mut_ptr().cast(),
        )
    };
    assert_ne!(handle2, core::ptr::null_mut());
    wit_bindgen_symmetric_rt::register(
        unsafe { EventSubscription::from_handle(handle2 as usize) },
        print_result,
        result2.as_mut_ptr().cast(),
    );

    let mut result3: [usize; 2] = [0, 0];
    let handle3 = unsafe {
        X5BasyncX5DtestX3AtestX2Fstring_delayX00forward(
            "C".as_ptr(),
            1,
            result3.as_mut_ptr().cast(),
        )
    };
    assert_ne!(handle3, core::ptr::null_mut());
    wit_bindgen_symmetric_rt::register(
        unsafe { EventSubscription::from_handle(handle3 as usize) },
        print_result,
        result3.as_mut_ptr().cast(),
    );

    wit_bindgen_symmetric_rt::run();
}
