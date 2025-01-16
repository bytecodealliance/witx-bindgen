use std::sync::atomic::{AtomicU32, Ordering};

use wit_bindgen_symmetric_rt::{
    async_support::Stream, register, CallbackState, EventSubscription,
};

static COUNT: AtomicU32 = AtomicU32::new(1);

extern "C" fn timer_call(data: *mut ()) -> CallbackState {
    let count = COUNT.fetch_add(1, Ordering::AcqRel);
    let stream: Stream = unsafe { Stream::from_handle(data as usize) };
    if count <= 5 {
        let buffer = stream.start_writing();
        let addr = buffer.get_address().take_handle() as *mut u32;
        let size = buffer.capacity();
        assert!(size >= 1);
        *unsafe { &mut *addr } = count;
        buffer.set_size(1);
        stream.finish_writing(Some(buffer));
    }
    let _ = stream.take_handle();
    CallbackState::Ready
}

extern "C" fn write_ready(data: *mut ()) -> CallbackState {
    let count = COUNT.load(Ordering::Acquire);
    if count > 5 {
        let stream: Stream = unsafe { Stream::from_handle(data as usize) };
        // EOF
        stream.finish_writing(None);
        CallbackState::Ready
    } else {
        if count == 1 {
            println!("we can write now, starting timer");
        }
        let ms_30 = EventSubscription::from_timeout(30 * 1_000_000);
        register(ms_30, timer_call, data);
        CallbackState::Pending
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub fn testX3AtestX2Fstream_sourceX00X5BasyncX5Dcreate(results: *mut u8) -> *mut u8 {
    let stream = Stream::new();
    let event = stream.write_ready_subscribe();
    let stream_copy = stream.clone();
    register(event, write_ready, stream_copy.take_handle() as *mut ());
    *unsafe { &mut *results.cast::<usize>() } = stream.take_handle();
    std::ptr::null_mut()
}
