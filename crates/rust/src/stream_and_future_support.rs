use {
    futures::{
        channel::oneshot,
        future::{self, FutureExt},
        sink::Sink,
        stream::Stream,
    },
    std::{
        collections::hash_map::Entry,
        convert::Infallible,
        fmt,
        future::{Future, IntoFuture},
        iter,
        marker::PhantomData,
        mem::{self, ManuallyDrop, MaybeUninit},
        pin::Pin,
        task::{Context, Poll},
    },
    wit_bindgen_rt::async_support::{self, Handle},
};

#[doc(hidden)]
pub trait FuturePayload: Unpin + Sized + 'static {
    fn new() -> u32;
    async fn write(future: u32, value: Self) -> bool;
    async fn read(future: u32) -> Option<Self>;
    fn cancel_write(future: u32);
    fn cancel_read(future: u32);
    fn close_writable(future: u32);
    fn close_readable(future: u32);
}

/// Represents the writable end of a Component Model `future`.
pub struct FutureWriter<T: FuturePayload> {
    handle: u32,
    _phantom: PhantomData<T>,
}

impl<T: FuturePayload> fmt::Debug for FutureWriter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FutureWriter")
            .field("handle", &self.handle)
            .finish()
    }
}

/// Represents a write operation which may be canceled prior to completion.
pub struct CancelableWrite<T: FuturePayload> {
    writer: Option<FutureWriter<T>>,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl<T: FuturePayload> Future for CancelableWrite<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
        let me = self.get_mut();
        match me.future.poll_unpin(cx) {
            Poll::Ready(()) => {
                me.writer = None;
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: FuturePayload> CancelableWrite<T> {
    /// Cancel this write if it hasn't already completed, returning the original `FutureWriter`.
    ///
    /// This method will panic if the write has already completed.
    pub fn cancel(mut self) -> FutureWriter<T> {
        self.cancel_mut()
    }

    fn cancel_mut(&mut self) -> FutureWriter<T> {
        let writer = self.writer.take().unwrap();
        async_support::with_entry(writer.handle, |entry| match entry {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::LocalOpen
                | Handle::LocalWaiting(_)
                | Handle::Read
                | Handle::LocalClosed => unreachable!(),
                Handle::LocalReady(..) => {
                    entry.insert(Handle::LocalOpen);
                }
                Handle::Write => T::cancel_write(writer.handle),
            },
        });
        writer
    }
}

impl<T: FuturePayload> Drop for CancelableWrite<T> {
    fn drop(&mut self) {
        if self.writer.is_some() {
            self.cancel_mut();
        }
    }
}

impl<T: FuturePayload> FutureWriter<T> {
    /// Write the specified value to this `future`.
    pub fn write(self, v: T) -> CancelableWrite<T> {
        let handle = self.handle;
        CancelableWrite {
            writer: Some(self),
            future: async_support::with_entry(handle, |entry| match entry {
                Entry::Vacant(_) => unreachable!(),
                Entry::Occupied(mut entry) => match entry.get() {
                    Handle::LocalOpen => {
                        let mut v = Some(v);
                        Box::pin(future::poll_fn(move |cx| {
                            async_support::with_entry(handle, |entry| match entry {
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
                        })) as Pin<Box<dyn Future<Output = _>>>
                    }
                    Handle::LocalWaiting(_) => {
                        let Handle::LocalWaiting(tx) = entry.insert(Handle::LocalClosed) else {
                            unreachable!()
                        };
                        _ = tx.send(Box::new(v));
                        Box::pin(future::ready(()))
                    }
                    Handle::LocalClosed => Box::pin(future::ready(())),
                    Handle::Read | Handle::LocalReady(..) => unreachable!(),
                    Handle::Write => Box::pin(T::write(handle, v).map(drop)),
                },
            }),
        }
    }
}

impl<T: FuturePayload> Drop for FutureWriter<T> {
    fn drop(&mut self) {
        async_support::with_entry(self.handle, |entry| match entry {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Handle::LocalOpen | Handle::LocalWaiting(_) | Handle::LocalReady(..) => {
                    entry.insert(Handle::LocalClosed);
                }
                Handle::Read => unreachable!(),
                Handle::Write | Handle::LocalClosed => {
                    entry.remove();
                    T::close_writable(self.handle);
                }
            },
        });
    }
}

/// Represents a read operation which may be canceled prior to completion.
pub struct CancelableRead<T: FuturePayload> {
    reader: Option<FutureReader<T>>,
    future: Pin<Box<dyn Future<Output = Option<T>>>>,
}

impl<T: FuturePayload> Future for CancelableRead<T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<T>> {
        let me = self.get_mut();
        match me.future.poll_unpin(cx) {
            Poll::Ready(v) => {
                me.reader = None;
                Poll::Ready(v)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: FuturePayload> CancelableRead<T> {
    /// Cancel this read if it hasn't already completed, returning the original `FutureReader`.
    ///
    /// This method will panic if the read has already completed.
    pub fn cancel(mut self) -> FutureReader<T> {
        self.cancel_mut()
    }

    fn cancel_mut(&mut self) -> FutureReader<T> {
        let reader = self.reader.take().unwrap();
        async_support::with_entry(reader.handle, |entry| match entry {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::LocalOpen
                | Handle::LocalReady(..)
                | Handle::Write
                | Handle::LocalClosed => unreachable!(),
                Handle::LocalWaiting(_) => {
                    entry.insert(Handle::LocalOpen);
                }
                Handle::Read => T::cancel_read(reader.handle),
            },
        });
        reader
    }
}

impl<T: FuturePayload> Drop for CancelableRead<T> {
    fn drop(&mut self) {
        if self.reader.is_some() {
            self.cancel_mut();
        }
    }
}

/// Represents the readable end of a Component Model `future`.
pub struct FutureReader<T: FuturePayload> {
    handle: u32,
    _phantom: PhantomData<T>,
}

impl<T: FuturePayload> fmt::Debug for FutureReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FutureReader")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<T: FuturePayload> FutureReader<T> {
    #[doc(hidden)]
    pub fn from_handle(handle: u32) -> Self {
        async_support::with_entry(handle, |entry| match entry {
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
        });

        Self {
            handle,
            _phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn into_handle(self) -> u32 {
        async_support::with_entry(self.handle, |entry| match entry {
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
        });

        ManuallyDrop::new(self).handle
    }
}

impl<T: FuturePayload> IntoFuture for FutureReader<T> {
    type Output = Option<T>;
    type IntoFuture = CancelableRead<T>;

    /// Convert this object into a `Future` which will resolve when a value is
    /// written to the writable end of this `future` (yielding a `Some` result)
    /// or when the writable end is dropped (yielding a `None` result).
    fn into_future(self) -> Self::IntoFuture {
        let handle = self.handle;
        CancelableRead {
            reader: Some(self),
            future: async_support::with_entry(handle, |entry| match entry {
                Entry::Vacant(_) => unreachable!(),
                Entry::Occupied(mut entry) => match entry.get() {
                    Handle::Write | Handle::LocalWaiting(_) => unreachable!(),
                    Handle::Read => Box::pin(async move { T::read(handle).await })
                        as Pin<Box<dyn Future<Output = _>>>,
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
            }),
        }
    }
}

impl<T: FuturePayload> Drop for FutureReader<T> {
    fn drop(&mut self) {
        async_support::with_entry(self.handle, |entry| match entry {
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
                    T::close_readable(self.handle);
                }
                Handle::Write => unreachable!(),
            },
        });
    }
}

#[doc(hidden)]
pub trait StreamPayload: Unpin + Sized + 'static {
    fn new() -> u32;
    async fn write(stream: u32, values: &[Self]) -> Option<usize>;
    async fn read(stream: u32, values: &mut [MaybeUninit<Self>]) -> Option<usize>;
    fn cancel_write(stream: u32);
    fn cancel_read(stream: u32);
    fn close_writable(stream: u32);
    fn close_readable(stream: u32);
}

struct CancelWriteOnDrop<T: StreamPayload> {
    handle: Option<u32>,
    _phantom: PhantomData<T>,
}

impl<T: StreamPayload> Drop for CancelWriteOnDrop<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            async_support::with_entry(handle, |entry| match entry {
                Entry::Vacant(_) => unreachable!(),
                Entry::Occupied(mut entry) => match entry.get() {
                    Handle::LocalOpen
                    | Handle::LocalWaiting(_)
                    | Handle::Read
                    | Handle::LocalClosed => unreachable!(),
                    Handle::LocalReady(..) => {
                        entry.insert(Handle::LocalOpen);
                    }
                    Handle::Write => T::cancel_write(handle),
                },
            });
        }
    }
}

/// Represents the writable end of a Component Model `stream`.
pub struct StreamWriter<T: StreamPayload> {
    handle: u32,
    future: Option<Pin<Box<dyn Future<Output = ()> + 'static>>>,
    _phantom: PhantomData<T>,
}

impl<T: StreamPayload> StreamWriter<T> {
    /// Cancel the current pending write operation.
    ///
    /// This will panic if no such operation is pending.
    pub fn cancel(&mut self) {
        assert!(self.future.is_some());
        self.future = None;
    }
}

impl<T: StreamPayload> fmt::Debug for StreamWriter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamWriter")
            .field("handle", &self.handle)
            .finish()
    }
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
        async_support::with_entry(self.handle, |entry| match entry {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get() {
                Handle::LocalOpen => {
                    let handle = self.handle;
                    let mut item = Some(item);
                    let mut cancel_on_drop = Some(CancelWriteOnDrop::<T> {
                        handle: Some(handle),
                        _phantom: PhantomData,
                    });
                    self.get_mut().future = Some(Box::pin(future::poll_fn(move |cx| {
                        async_support::with_entry(handle, |entry| match entry {
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
                                        cancel_on_drop.take().unwrap().handle = None;
                                        Poll::Ready(())
                                    }
                                }
                                Handle::LocalReady(..) => Poll::Pending,
                                Handle::LocalClosed => {
                                    cancel_on_drop.take().unwrap().handle = None;
                                    Poll::Ready(())
                                }
                                Handle::LocalWaiting(_) | Handle::Read | Handle::Write => {
                                    unreachable!()
                                }
                            },
                        })
                    })));
                }
                Handle::LocalWaiting(_) => {
                    let Handle::LocalWaiting(tx) = entry.insert(Handle::LocalOpen) else {
                        unreachable!()
                    };
                    _ = tx.send(Box::new(item));
                }
                Handle::LocalClosed => (),
                Handle::Read | Handle::LocalReady(..) => unreachable!(),
                Handle::Write => {
                    let handle = self.handle;
                    let mut cancel_on_drop = CancelWriteOnDrop::<T> {
                        handle: Some(handle),
                        _phantom: PhantomData,
                    };
                    self.get_mut().future = Some(Box::pin(async move {
                        let mut offset = 0;
                        while offset < item.len() {
                            if let Some(count) = T::write(handle, &item[offset..]).await {
                                offset += count;
                            } else {
                                break;
                            }
                        }
                        cancel_on_drop.handle = None;
                        drop(cancel_on_drop);
                    }));
                }
            },
        });
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
        self.future = None;

        async_support::with_entry(self.handle, |entry| match entry {
            Entry::Vacant(_) => unreachable!(),
            Entry::Occupied(mut entry) => match entry.get_mut() {
                Handle::LocalOpen | Handle::LocalWaiting(_) | Handle::LocalReady(..) => {
                    entry.insert(Handle::LocalClosed);
                }
                Handle::Read => unreachable!(),
                Handle::Write | Handle::LocalClosed => {
                    entry.remove();
                    T::close_writable(self.handle);
                }
            },
        });
    }
}

struct CancelReadOnDrop<T: StreamPayload> {
    handle: Option<u32>,
    _phantom: PhantomData<T>,
}

impl<T: StreamPayload> Drop for CancelReadOnDrop<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            async_support::with_entry(handle, |entry| match entry {
                Entry::Vacant(_) => unreachable!(),
                Entry::Occupied(mut entry) => match entry.get() {
                    Handle::LocalOpen
                    | Handle::LocalReady(..)
                    | Handle::Write
                    | Handle::LocalClosed => unreachable!(),
                    Handle::LocalWaiting(_) => {
                        entry.insert(Handle::LocalOpen);
                    }
                    Handle::Read => T::cancel_read(handle),
                },
            });
        }
    }
}

/// Represents the readable end of a Component Model `stream`.
pub struct StreamReader<T: StreamPayload> {
    handle: u32,
    future: Option<Pin<Box<dyn Future<Output = Option<Vec<T>>> + 'static>>>,
    _phantom: PhantomData<T>,
}

impl<T: StreamPayload> StreamReader<T> {
    /// Cancel the current pending read operation.
    ///
    /// This will panic if no such operation is pending.
    pub fn cancel(&mut self) {
        assert!(self.future.is_some());
        self.future = None;
    }
}

impl<T: StreamPayload> fmt::Debug for StreamReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamReader")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<T: StreamPayload> StreamReader<T> {
    #[doc(hidden)]
    pub fn from_handle(handle: u32) -> Self {
        async_support::with_entry(handle, |entry| match entry {
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
        });

        Self {
            handle,
            future: None,
            _phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn into_handle(self) -> u32 {
        async_support::with_entry(self.handle, |entry| match entry {
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
        });

        ManuallyDrop::new(self).handle
    }
}

impl<T: StreamPayload> Stream for StreamReader<T> {
    type Item = Vec<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();

        if me.future.is_none() {
            me.future = Some(async_support::with_entry(me.handle, |entry| match entry {
                Entry::Vacant(_) => unreachable!(),
                Entry::Occupied(mut entry) => match entry.get() {
                    Handle::Write | Handle::LocalWaiting(_) => unreachable!(),
                    Handle::Read => {
                        let handle = me.handle;
                        let mut cancel_on_drop = CancelReadOnDrop::<T> {
                            handle: Some(handle),
                            _phantom: PhantomData,
                        };
                        Box::pin(async move {
                            let mut buffer = iter::repeat_with(MaybeUninit::uninit)
                                .take(ceiling(64 * 1024, mem::size_of::<T>()))
                                .collect::<Vec<_>>();

                            let result = if let Some(count) = T::read(handle, &mut buffer).await {
                                buffer.truncate(count);
                                Some(unsafe {
                                    mem::transmute::<Vec<MaybeUninit<T>>, Vec<T>>(buffer)
                                })
                            } else {
                                None
                            };
                            cancel_on_drop.handle = None;
                            drop(cancel_on_drop);
                            result
                        }) as Pin<Box<dyn Future<Output = _>>>
                    }
                    Handle::LocalOpen => {
                        let (tx, rx) = oneshot::channel();
                        entry.insert(Handle::LocalWaiting(tx));
                        let mut cancel_on_drop = CancelReadOnDrop::<T> {
                            handle: Some(me.handle),
                            _phantom: PhantomData,
                        };
                        Box::pin(async move {
                            let result = rx.map(|v| v.ok().map(|v| *v.downcast().unwrap())).await;
                            cancel_on_drop.handle = None;
                            drop(cancel_on_drop);
                            result
                        })
                    }
                    Handle::LocalClosed => Box::pin(future::ready(None)),
                    Handle::LocalReady(..) => {
                        let Handle::LocalReady(v, waker) = entry.insert(Handle::LocalOpen) else {
                            unreachable!()
                        };
                        waker.wake();
                        Box::pin(future::ready(Some(*v.downcast().unwrap())))
                    }
                },
            }));
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
        self.future = None;

        async_support::with_entry(self.handle, |entry| match entry {
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
                    T::close_readable(self.handle);
                }
                Handle::Write => unreachable!(),
            },
        });
    }
}

/// Creates a new Component Model `future` with the specified payload type.
pub fn new_future<T: FuturePayload>() -> (FutureWriter<T>, FutureReader<T>) {
    let handle = T::new();
    async_support::with_entry(handle, |entry| match entry {
        Entry::Vacant(entry) => {
            entry.insert(Handle::LocalOpen);
        }
        Entry::Occupied(_) => unreachable!(),
    });
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

/// Creates a new Component Model `stream` with the specified payload type.
pub fn new_stream<T: StreamPayload>() -> (StreamWriter<T>, StreamReader<T>) {
    let handle = T::new();
    async_support::with_entry(handle, |entry| match entry {
        Entry::Vacant(entry) => {
            entry.insert(Handle::LocalOpen);
        }
        Entry::Occupied(_) => unreachable!(),
    });
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

fn ceiling(x: usize, y: usize) -> usize {
    (x / y) + if x % y == 0 { 0 } else { 1 }
}
