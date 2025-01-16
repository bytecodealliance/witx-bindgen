pub use crate::module::symmetric::runtime::symmetric_stream::StreamObj as Stream;
use crate::{
    async_support::wait_on,
    symmetric_stream::{Address, Buffer},
};
use {
    futures::sink::Sink,
    std::{
        convert::Infallible,
        fmt,
        future::Future,
        iter,
        marker::PhantomData,
        mem::{self, MaybeUninit},
        pin::Pin,
        task::{Context, Poll},
    },
};

fn ceiling(x: usize, y: usize) -> usize {
    (x / y) + if x % y == 0 { 0 } else { 1 }
}

pub mod results {
    pub const BLOCKED: isize = -1;
    pub const CLOSED: isize = isize::MIN;
    pub const CANCELED: isize = 0;
}

pub struct StreamWriter<T: 'static> {
    handle: StreamHandle2,
    future: Option<Pin<Box<dyn Future<Output = ()> + 'static + Send>>>,
    _phantom: PhantomData<T>,
}

impl<T> StreamWriter<T> {
    #[doc(hidden)]
    pub fn new(handle: Stream) -> Self {
        Self {
            handle,
            future: None,
            _phantom: PhantomData,
        }
    }

    pub fn cancel(&mut self) {
        todo!()
    }
}

impl<T> fmt::Debug for StreamWriter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamWriter")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<T: Unpin> Sink<Vec<T>> for StreamWriter<T> {
    type Error = Infallible;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let me = self.get_mut();

        let ready = me.handle.is_ready_to_write();

        // see also StreamReader::poll_next
        if !ready && me.future.is_none() {
            let handle = me.handle.clone();
            me.future = Some(Box::pin(async move {
                let handle_local = handle;
                let subscr = handle_local.write_ready_subscribe();
                subscr.reset();
                wait_on(subscr).await;
            }) as Pin<Box<dyn Future<Output = _> + Send>>);
        }

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

    fn start_send(self: Pin<&mut Self>, mut item: Vec<T>) -> Result<(), Self::Error> {
        let item_len = item.len();
        let me = self.get_mut();
        let stream = &me.handle;
        let buffer = stream.start_writing();
        let addr = buffer.get_address().take_handle() as *mut u8;
        let size = buffer.capacity() as usize;
        assert!(size >= item_len);
        let slice =
            unsafe { std::slice::from_raw_parts_mut(addr.cast::<MaybeUninit<T>>(), item_len) };
        for (a, b) in slice.iter_mut().zip(item.drain(..)) {
            a.write(b);
        }
        buffer.set_size(item_len as u64);
        stream.finish_writing(Some(buffer));
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_ready(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.poll_ready(cx)
    }
}

impl<T> Drop for StreamWriter<T> {
    fn drop(&mut self) {
        if !self.handle.is_write_closed() {
            self.handle.finish_writing(None);
        }
    }
}

/// Represents the readable end of a Component Model `stream`.
pub struct StreamReader<T: 'static> {
    handle: StreamHandle2,
    future: Option<Pin<Box<dyn Future<Output = Option<Vec<T>>> + 'static + Send>>>,
    _phantom: PhantomData<T>,
}

impl<T> StreamReader<T> {
    /// Cancel the current pending read operation.
    ///
    /// This will panic if no such operation is pending.
    pub fn cancel(&mut self) {
        assert!(self.future.is_some());
        self.future = None;
    }
}

impl<T> fmt::Debug for StreamReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamReader")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<T> StreamReader<T> {
    #[doc(hidden)]
    pub fn new(handle: Stream) -> Self {
        Self {
            handle,
            future: None,
            _phantom: PhantomData,
        }
    }
    #[doc(hidden)]
    pub fn into_handle(self) -> *mut () {
        self.handle.take_handle() as *mut ()
    }
}

impl<T: Unpin + Send> futures::stream::Stream for StreamReader<T> {
    type Item = Vec<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();

        if me.future.is_none() {
            let handle = me.handle.clone();
            me.future = Some(Box::pin(async move {
                let mut buffer0 = iter::repeat_with(MaybeUninit::uninit)
                    .take(ceiling(4 * 1024, mem::size_of::<T>()))
                    .collect::<Vec<_>>();
                let address = unsafe { Address::from_handle(buffer0.as_mut_ptr() as usize) };
                let buffer = Buffer::new(address, buffer0.len() as u64);
                handle.start_reading(buffer);
                let subsc = handle.read_ready_subscribe();
                subsc.reset();
                wait_on(subsc).await;
                let buffer2 = handle.read_result();
                if let Some(buffer2) = buffer2 {
                    let count = buffer2.get_size();
                    buffer0.truncate(count as usize);
                    Some(unsafe { mem::transmute::<Vec<MaybeUninit<T>>, Vec<T>>(buffer0) })
                } else {
                    None
                }
            }) as Pin<Box<dyn Future<Output = _> + Send>>);
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

impl<T> Drop for StreamReader<T> {
    fn drop(&mut self) {
        if self.handle.handle() != 0 {
            self.handle.write_ready_activate();
        }
    }
}

pub type StreamHandle2 = Stream;

pub fn new_stream<T: 'static>() -> (StreamWriter<T>, StreamReader<T>) {
    let handle = Stream::new();
    let handle2 = handle.clone();
    (StreamWriter::new(handle), StreamReader::new(handle2))
}
