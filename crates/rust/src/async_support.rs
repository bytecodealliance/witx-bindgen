use {
    futures::{
        channel::oneshot,
        future::{self, FutureExt},
        sink::Sink,
        stream::{FuturesUnordered, Stream, StreamExt},
    },
    once_cell::sync::Lazy,
    std::{
        alloc::{self, Layout},
        any::Any,
        collections::hash_map::Entry,
        collections::HashMap,
        convert::Infallible,
        fmt::{self, Debug, Display},
        future::{Future, IntoFuture},
        iter,
        marker::PhantomData,
        mem::{self, ManuallyDrop, MaybeUninit},
        pin::{pin, Pin},
        ptr,
        sync::Arc,
        task::{Context, Poll, Wake, Waker},
    },
};

type BoxFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;

struct FutureState {
    todo: usize,
    tasks: Option<FuturesUnordered<BoxFuture>>,
}

static mut CURRENT: *mut FutureState = ptr::null_mut();

static mut CALLS: Lazy<HashMap<i32, oneshot::Sender<u32>>> = Lazy::new(HashMap::new);

static mut SPAWNED: Vec<BoxFuture> = Vec::new();

enum Handle {
    LocalOpen,
    LocalReady(Box<dyn Any>, Waker),
    LocalWaiting(oneshot::Sender<Box<dyn Any>>),
    LocalClosed,
    Read,
    Write,
}

static mut HANDLES: Lazy<HashMap<u32, Handle>> = Lazy::new(HashMap::new);

fn dummy_waker() -> Waker {
    struct DummyWaker;

    impl Wake for DummyWaker {
        fn wake(self: Arc<Self>) {}
    }

    static WAKER: Lazy<Arc<DummyWaker>> = Lazy::new(|| Arc::new(DummyWaker));

    WAKER.clone().into()
}

unsafe fn poll(state: *mut FutureState) -> Poll<()> {
    loop {
        if let Some(futures) = (*state).tasks.as_mut() {
            CURRENT = state;
            let poll = futures.poll_next_unpin(&mut Context::from_waker(&dummy_waker()));
            CURRENT = ptr::null_mut();

            if SPAWNED.is_empty() {
                match poll {
                    Poll::Ready(Some(())) => (),
                    Poll::Ready(None) => {
                        (*state).tasks = None;
                        break Poll::Ready(());
                    }
                    Poll::Pending => break Poll::Pending,
                }
            } else {
                futures.extend(SPAWNED.drain(..));
            }
        } else {
            break Poll::Ready(());
        }
    }
}

pub fn first_poll<T: 'static>(
    future: impl Future<Output = T> + 'static,
    fun: impl FnOnce(T) + 'static,
) -> *mut u8 {
    let state = Box::into_raw(Box::new(FutureState {
        todo: 0,
        tasks: Some(
            [Box::pin(future.map(fun)) as BoxFuture]
                .into_iter()
                .collect(),
        ),
    }));
    match unsafe { poll(state) } {
        Poll::Ready(()) => ptr::null_mut(),
        Poll::Pending => state as _,
    }
}

pub async unsafe fn await_result(
    import: unsafe extern "C" fn(*mut u8, *mut u8) -> i32,
    params_layout: Layout,
    params: *mut u8,
    results: *mut u8,
) {
    const STATUS_STARTING: u32 = 0;
    const STATUS_STARTED: u32 = 1;
    const STATUS_RETURNED: u32 = 2;
    const STATUS_DONE: u32 = 3;

    let result = import(params, results) as u32;
    let status = result >> 30;
    let call = (result & !(0b11 << 30)) as i32;

    if status != STATUS_DONE {
        assert!(!CURRENT.is_null());
        (*CURRENT).todo += 1;
    }

    match status {
        STATUS_STARTING => {
            let (tx, rx) = oneshot::channel();
            CALLS.insert(call, tx);
            rx.await.unwrap();
            alloc::dealloc(params, params_layout);
        }
        STATUS_STARTED => {
            alloc::dealloc(params, params_layout);
            let (tx, rx) = oneshot::channel();
            CALLS.insert(call, tx);
            rx.await.unwrap();
        }
        STATUS_RETURNED | STATUS_DONE => {
            alloc::dealloc(params, params_layout);
        }
        _ => unreachable!(),
    }
}

mod results {
    pub const BLOCKED: u32 = 0xffff_ffff;
    pub const CLOSED: u32 = 0x8000_0000;
    pub const CANCELED: u32 = 0;
}

pub async unsafe fn await_future_result(
    import: unsafe extern "C" fn(u32, *mut u8) -> u32,
    future: u32,
    address: *mut u8,
) -> bool {
    let result = import(future, address);
    match result {
        results::BLOCKED => {
            assert!(!CURRENT.is_null());
            (*CURRENT).todo += 1;
            let (tx, rx) = oneshot::channel();
            CALLS.insert(future as _, tx);
            let v = rx.await.unwrap();
            v == 1
        }
        results::CLOSED | results::CANCELED => false,
        1 => true,
        _ => unreachable!(),
    }
}

pub async unsafe fn await_stream_result(
    import: unsafe extern "C" fn(u32, *mut u8, u32) -> u32,
    stream: u32,
    address: *mut u8,
    count: u32,
) -> Option<usize> {
    let result = import(stream, address, count);
    match result {
        results::BLOCKED => {
            assert!(!CURRENT.is_null());
            (*CURRENT).todo += 1;
            let (tx, rx) = oneshot::channel();
            CALLS.insert(stream as _, tx);
            let v = rx.await.unwrap();
            if let results::CLOSED | results::CANCELED = v {
                None
            } else {
                Some(usize::try_from(v).unwrap())
            }
        }
        results::CLOSED | results::CANCELED => None,
        v => Some(usize::try_from(v).unwrap()),
    }
}

fn subtask_drop(subtask: u32) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "[subtask-drop]"]
            fn subtask_drop(_: u32);
        }
        unsafe {
            subtask_drop(subtask);
        }
    }
}

pub unsafe fn callback(ctx: *mut u8, event0: i32, event1: i32, event2: i32) -> i32 {
    const EVENT_CALL_STARTING: i32 = 0;
    const EVENT_CALL_STARTED: i32 = 1;
    const EVENT_CALL_RETURNED: i32 = 2;
    const EVENT_CALL_DONE: i32 = 3;
    const EVENT_YIELDED: i32 = 4;
    const EVENT_STREAM_READ: i32 = 5;
    const EVENT_STREAM_WRITE: i32 = 6;
    const EVENT_FUTURE_READ: i32 = 7;
    const EVENT_FUTURE_WRITE: i32 = 8;

    match event0 {
        EVENT_CALL_STARTED => {
            // TODO: could dealloc params here if we attached the pointer to the call
            0
        }
        EVENT_CALL_RETURNED | EVENT_CALL_DONE | EVENT_STREAM_READ | EVENT_STREAM_WRITE
        | EVENT_FUTURE_READ | EVENT_FUTURE_WRITE => {
            if let Some(call) = CALLS.remove(&event1) {
                call.send(event2 as _);
            }

            let state = ctx as *mut FutureState;
            let done = poll(state).is_ready();

            if event0 == EVENT_CALL_DONE {
                subtask_drop(event1 as u32);
            }

            if matches!(
                event0,
                EVENT_CALL_DONE
                    | EVENT_STREAM_READ
                    | EVENT_STREAM_WRITE
                    | EVENT_FUTURE_READ
                    | EVENT_FUTURE_WRITE
            ) {
                (*state).todo -= 1;
            }

            if done && (*state).todo == 0 {
                drop(Box::from_raw(state));
                1
            } else {
                0
            }
        }
        _ => unreachable!(),
    }
}

#[doc(hidden)]
pub trait FuturePayload: Sized + 'static {
    fn new() -> u32;
    async fn write(future: u32, value: Self) -> bool;
    async fn read(future: u32) -> Option<Self>;
    fn drop_writer(future: u32);
    fn drop_reader(future: u32);
}

pub struct FutureWriter<T: FuturePayload> {
    handle: u32,
    _phantom: PhantomData<T>,
}

impl<T: FuturePayload> FutureWriter<T> {
    pub async fn write(self, v: T) {
        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::LocalOpen => {
                    let mut v = Some(v);
                    future::poll_fn(move |cx| match unsafe { HANDLES.entry(self.handle) } {
                        Entry::Vacant(_) => unreachable!(),
                        Entry::Occupied(mut entry) => match entry.get() {
                            Handle::LocalOpen => {
                                entry.insert(Handle::LocalReady(
                                    Box::new(v.take().unwrap()),
                                    cx.waker().clone(),
                                ));
                                Poll::Pending
                            }
                            Handle::LocalReady(..) => Poll::Pending,
                            Handle::LocalClosed => Poll::Ready(()),
                            Handle::LocalWaiting(_) | Handle::Read | Handle::Write => {
                                unreachable!()
                            }
                        },
                    })
                    .await
                }
                Handle::LocalWaiting(_) => {
                    let Handle::LocalWaiting(tx) = entry.insert(Handle::LocalClosed) else {
                        unreachable!()
                    };
                    tx.send(Box::new(v));
                }
                Handle::LocalClosed => (),
                Handle::Read | Handle::LocalReady(..) => unreachable!(),
                Handle::Write => {
                    T::write(self.handle, v).await;
                }
            },
        }
    }
}

impl<T: FuturePayload> Drop for FutureWriter<T> {
    fn drop(&mut self) {
        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Handle::LocalOpen | Handle::LocalWaiting(_) | Handle::LocalReady(..) => {
                    entry.insert(Handle::LocalClosed);
                }
                Handle::Read => unreachable!(),
                Handle::Write | Handle::LocalClosed => {
                    entry.remove();
                    T::drop_writer(self.handle);
                }
            },
        }
    }
}

pub struct FutureReader<T: FuturePayload> {
    handle: u32,
    _phantom: PhantomData<T>,
}

impl<T: FuturePayload> FutureReader<T> {
    #[doc(hidden)]
    pub fn from_handle(handle: u32) -> Self {
        match unsafe { HANDLES.entry(handle) } {
            Entry::Vacant(entry) => {
                entry.insert(Handle::Read);
            }
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::Write => {
                    entry.insert(Handle::LocalOpen);
                }
                Handle::Read
                | Handle::LocalOpen
                | Handle::LocalReady(..)
                | Handle::LocalWaiting(_)
                | Handle::LocalClosed => {
                    unreachable!()
                }
            },
        }

        Self {
            handle,
            _phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn into_handle(self) -> u32 {
        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::LocalOpen => {
                    entry.insert(Handle::Write);
                }
                Handle::Read | Handle::LocalClosed => {
                    entry.remove();
                }
                Handle::LocalReady(..) | Handle::LocalWaiting(_) | Handle::Write => unreachable!(),
            },
        }

        ManuallyDrop::new(self).handle
    }
}

impl<T: FuturePayload> IntoFuture for FutureReader<T> {
    type Output = Option<T>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::Write | Handle::LocalWaiting(_) => unreachable!(),
                Handle::Read => Box::pin(async move { T::read(self.handle).await }),
                Handle::LocalOpen => {
                    let (tx, rx) = oneshot::channel();
                    entry.insert(Handle::LocalWaiting(tx));
                    Box::pin(async move { rx.await.ok().map(|v| *v.downcast().unwrap()) })
                }
                Handle::LocalClosed => Box::pin(future::ready(None)),
                Handle::LocalReady(..) => {
                    let Handle::LocalReady(v, waker) = entry.insert(Handle::LocalClosed) else {
                        unreachable!()
                    };
                    waker.wake();
                    Box::pin(future::ready(Some(*v.downcast().unwrap())))
                }
            },
        }
    }
}

impl<T: FuturePayload> Drop for FutureReader<T> {
    fn drop(&mut self) {
        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Handle::LocalReady(..) => {
                    let Handle::LocalReady(_, waker) = entry.insert(Handle::LocalClosed) else {
                        unreachable!()
                    };
                    waker.wake();
                }
                Handle::LocalOpen | Handle::LocalWaiting(_) => {
                    entry.insert(Handle::LocalClosed);
                }
                Handle::Read | Handle::LocalClosed => {
                    entry.remove();
                    T::drop_reader(self.handle);
                }
                Handle::Write => unreachable!(),
            },
        }
    }
}

#[doc(hidden)]
pub trait StreamPayload: Unpin + Sized + 'static {
    fn new() -> u32;
    async fn write(stream: u32, values: &[Self]) -> Option<usize>;
    async fn read(stream: u32, values: &mut [MaybeUninit<Self>]) -> Option<usize>;
    fn drop_writer(future: u32);
    fn drop_reader(future: u32);
}

pub struct StreamWriter<T: StreamPayload> {
    handle: u32,
    future: Option<Pin<Box<dyn Future<Output = ()> + 'static>>>,
    _phantom: PhantomData<T>,
}

impl<T: StreamPayload> Sink<Vec<T>> for StreamWriter<T> {
    type Error = Infallible;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let me = self.get_mut();

        if let Some(future) = &mut me.future {
            match future.as_mut().poll(cx) {
                Poll::Ready(_) => {
                    me.future = None;
                    Poll::Ready(Ok(()))
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Vec<T>) -> Result<(), Self::Error> {
        assert!(self.future.is_none());
        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::LocalOpen => {
                    let handle = self.handle;
                    let mut item = Some(item);
                    self.get_mut().future = Some(Box::pin(future::poll_fn(move |cx| {
                        match unsafe { HANDLES.entry(handle) } {
                            Entry::Vacant(_) => unreachable!(),
                            Entry::Occupied(mut entry) => match entry.get() {
                                Handle::LocalOpen => {
                                    if let Some(item) = item.take() {
                                        entry.insert(Handle::LocalReady(
                                            Box::new(item),
                                            cx.waker().clone(),
                                        ));
                                        Poll::Pending
                                    } else {
                                        Poll::Ready(())
                                    }
                                }
                                Handle::LocalReady(..) => Poll::Pending,
                                Handle::LocalClosed => Poll::Ready(()),
                                Handle::LocalWaiting(_) | Handle::Read | Handle::Write => {
                                    unreachable!()
                                }
                            },
                        }
                    })));
                }
                Handle::LocalWaiting(_) => {
                    let Handle::LocalWaiting(tx) = entry.insert(Handle::LocalOpen) else {
                        unreachable!()
                    };
                    tx.send(Box::new(item));
                }
                Handle::LocalClosed => (),
                Handle::Read | Handle::LocalReady(..) => unreachable!(),
                Handle::Write => {
                    let handle = self.handle;
                    self.get_mut().future = Some(Box::pin(async move {
                        let mut offset = 0;
                        while offset < item.len() {
                            if let Some(count) = T::write(handle, &item[offset..]).await {
                                offset += count;
                            } else {
                                break;
                            }
                        }
                    }));
                }
            },
        }
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_ready(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_ready(cx)
    }
}

impl<T: StreamPayload> Drop for StreamWriter<T> {
    fn drop(&mut self) {
        if self.future.is_some() {
            todo!("gracefully handle `StreamWriter::drop` when a write is in progress");
        }

        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Handle::LocalOpen | Handle::LocalWaiting(_) | Handle::LocalReady(..) => {
                    entry.insert(Handle::LocalClosed);
                }
                Handle::Read => unreachable!(),
                Handle::Write | Handle::LocalClosed => {
                    entry.remove();
                    T::drop_writer(self.handle);
                }
            },
        }
    }
}

pub struct StreamReader<T: StreamPayload> {
    handle: u32,
    future: Option<Pin<Box<dyn Future<Output = Option<Vec<T>>> + 'static>>>,
    _phantom: PhantomData<T>,
}

impl<T: StreamPayload> StreamReader<T> {
    #[doc(hidden)]
    pub fn from_handle(handle: u32) -> Self {
        match unsafe { HANDLES.entry(handle) } {
            Entry::Vacant(mut entry) => {
                entry.insert(Handle::Read);
            }
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::Write => {
                    entry.insert(Handle::LocalOpen);
                }
                Handle::Read
                | Handle::LocalOpen
                | Handle::LocalReady(..)
                | Handle::LocalWaiting(_)
                | Handle::LocalClosed => {
                    unreachable!()
                }
            },
        }

        Self {
            handle,
            future: None,
            _phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn into_handle(self) -> u32 {
        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::LocalOpen => {
                    entry.insert(Handle::Write);
                }
                Handle::Read | Handle::LocalClosed => {
                    entry.remove();
                }
                Handle::LocalReady(..) | Handle::LocalWaiting(_) | Handle::Write => unreachable!(),
            },
        }

        ManuallyDrop::new(self).handle
    }
}

impl<T: StreamPayload> Stream for StreamReader<T> {
    type Item = Vec<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();

        if me.future.is_none() {
            me.future = Some(match unsafe { HANDLES.entry(me.handle) } {
                Entry::Vacant(_) => unreachable!(),
                Entry::Occupied(mut entry) => match entry.get() {
                    Handle::Write | Handle::LocalWaiting(_) => unreachable!(),
                    Handle::Read => {
                        let handle = me.handle;
                        Box::pin(async move {
                            let mut buffer = iter::repeat_with(MaybeUninit::uninit)
                                .take(ceiling(64 * 1024, mem::size_of::<T>()))
                                .collect::<Vec<_>>();

                            if let Some(count) = T::read(handle, &mut buffer).await {
                                buffer.truncate(count);
                                Some(unsafe {
                                    mem::transmute::<Vec<MaybeUninit<T>>, Vec<T>>(buffer)
                                })
                            } else {
                                None
                            }
                        })
                    }
                    Handle::LocalOpen => {
                        let (tx, rx) = oneshot::channel();
                        entry.insert(Handle::LocalWaiting(tx));
                        Box::pin(rx.map(|v| v.ok().map(|v| *v.downcast().unwrap())))
                    }
                    Handle::LocalClosed => return Poll::Ready(None),
                    Handle::LocalReady(..) => {
                        let Handle::LocalReady(v, waker) = entry.insert(Handle::LocalOpen) else {
                            unreachable!()
                        };
                        waker.wake();
                        return Poll::Ready(Some(*v.downcast().unwrap()));
                    }
                },
            });
        }

        match me.future.as_mut().unwrap().as_mut().poll(cx) {
            Poll::Ready(v) => {
                me.future = None;
                Poll::Ready(v)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: StreamPayload> Drop for StreamReader<T> {
    fn drop(&mut self) {
        if self.future.is_some() {
            todo!("gracefully handle `StreamReader::drop` when a read is in progress");
        }

        match unsafe { HANDLES.entry(self.handle) } {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Handle::LocalReady(..) => {
                    let Handle::LocalReady(_, waker) = entry.insert(Handle::LocalClosed) else {
                        unreachable!()
                    };
                    waker.wake();
                }
                Handle::LocalOpen | Handle::LocalWaiting(_) => {
                    entry.insert(Handle::LocalClosed);
                }
                Handle::Read | Handle::LocalClosed => {
                    entry.remove();
                    T::drop_reader(self.handle);
                }
                Handle::Write => unreachable!(),
            },
        }
    }
}

pub struct Error {
    handle: u32,
}

impl Error {
    #[doc(hidden)]
    pub fn from_handle(handle: u32) -> Self {
        Self { handle }
    }

    #[doc(hidden)]
    pub fn handle(&self) -> u32 {
        self.handle
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error").finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error")
    }
}

impl std::error::Error for Error {}

impl Drop for Error {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            unreachable!();
        }

        #[cfg(target_arch = "wasm32")]
        {
            #[link(wasm_import_module = "$root")]
            extern "C" {
                #[link_name = "[error-drop]"]
                fn error_drop(_: u32);
            }
            if self.handle != 0 {
                unsafe { error_drop(self.handle) }
            }
        }
    }
}

pub fn new_future<T: FuturePayload>() -> (FutureWriter<T>, FutureReader<T>) {
    let handle = T::new();
    unsafe { HANDLES.insert(handle, Handle::LocalOpen) };
    (
        FutureWriter {
            handle,
            _phantom: PhantomData,
        },
        FutureReader {
            handle,
            _phantom: PhantomData,
        },
    )
}

pub fn new_stream<T: StreamPayload>() -> (StreamWriter<T>, StreamReader<T>) {
    let handle = T::new();
    unsafe { HANDLES.insert(handle, Handle::LocalOpen) };
    (
        StreamWriter {
            handle,
            future: None,
            _phantom: PhantomData,
        },
        StreamReader {
            handle,
            future: None,
            _phantom: PhantomData,
        },
    )
}

pub fn spawn(future: impl Future<Output = ()> + 'static) {
    unsafe { SPAWNED.push(Box::pin(future)) }
}

fn task_wait(state: &mut FutureState) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "[task-wait]"]
            fn wait(_: *mut i32) -> i32;
        }
        let mut payload = [0i32; 2];
        unsafe {
            let event0 = wait(payload.as_mut_ptr());
            callback(state as *mut _ as _, event0, payload[0], payload[1]);
        }
    }
}

// TODO: refactor so `'static` bounds aren't necessary
pub fn block_on<T: 'static>(future: impl Future<Output = T> + 'static) -> T {
    let (mut tx, mut rx) = oneshot::channel();
    let state = &mut FutureState {
        todo: 0,
        tasks: Some(
            [Box::pin(future.map(move |v| drop(tx.send(v)))) as BoxFuture]
                .into_iter()
                .collect(),
        ),
    };
    loop {
        match unsafe { poll(state) } {
            Poll::Ready(()) => break rx.try_recv().unwrap().unwrap(),
            Poll::Pending => task_wait(state),
        }
    }
}

fn task_poll(state: &mut FutureState) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "[task-poll]"]
            fn poll(_: *mut i32) -> i32;
        }
        let mut payload = [0i32; 3];
        unsafe {
            let got_event = poll(payload.as_mut_ptr()) != 0;
            if got_event {
                callback(state as *mut _ as _, payload[0], payload[1], payload[2]);
            }
            got_event
        }
    }
}

// TODO: refactor so `'static` bounds aren't necessary
pub fn poll_future<T: 'static>(future: impl Future<Output = T> + 'static) -> Option<T> {
    let (mut tx, mut rx) = oneshot::channel();
    let state = &mut FutureState {
        todo: 0,
        tasks: Some(
            [Box::pin(future.map(move |v| drop(tx.send(v)))) as BoxFuture]
                .into_iter()
                .collect(),
        ),
    };
    loop {
        match unsafe { poll(state) } {
            Poll::Ready(()) => break Some(rx.try_recv().unwrap().unwrap()),
            Poll::Pending => {
                if !task_poll(state) {
                    break None;
                }
            }
        }
    }
}

pub fn task_yield() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "[task-yield]"]
            fn yield_();
        }
        unsafe {
            yield_();
        }
    }
}

pub fn task_backpressure(enabled: bool) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    {
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "[task-backpressure]"]
            fn backpressure(_: i32);
        }
        unsafe {
            backpressure(if enabled { 1 } else { 0 });
        }
    }
}

fn ceiling(x: usize, y: usize) -> usize {
    (x / y) + if x % y == 0 { 0 } else { 1 }
}
