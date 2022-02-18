use crate::*;
use futures::Stream;
#[allow(unused)]
pub mod api {
    use core::future::Future;
    use core::mem::ManuallyDrop;
    use core::pin::Pin;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use std::sync::Arc;
    use std::ffi::c_void;
    use super::*;

    #[doc=" Try to execute some function, catching any panics and aborting to make sure Rust"]
    #[doc=" doesn't unwind across the FFI boundary."]
    pub fn panic_abort<R>(func: impl FnOnce() -> R + std::panic::UnwindSafe) -> R {
        match std::panic::catch_unwind(func) {
            Ok(res) => res,
            Err(_) => {
                std::process::abort();
            }
        }
    }

    #[inline(always)]
    pub fn assert_send_static<T: Send + 'static>(t: T) -> T {
        t
    }

    pub type Result<T, E = String> = core::result::Result<T, E>;

    #[no_mangle]
    pub unsafe extern "C" fn allocate(size: usize, align: usize) -> *mut u8 {
        let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
        let ptr = std::alloc::alloc(layout);
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        ptr
    }

    #[no_mangle]
    pub unsafe extern "C" fn deallocate(ptr: *mut u8, size: usize, align: usize) {
        let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
        std::alloc::dealloc(ptr, layout);
    }

    pub struct FfiBuffer<T> {
        addr: usize,
        size: usize,
        alloc: usize,
        phantom: std::marker::PhantomData<T>
    }

    impl<T> FfiBuffer<T> {
        pub fn new(data: Vec<T>) -> FfiBuffer<T> {
            unsafe {
                let (addr, size, alloc) = data.into_raw_parts();
                FfiBuffer {
                    addr: std::mem::transmute(addr),
                    size: size * std::mem::size_of::<T>(),
                    alloc: alloc * std::mem::size_of::<T>(),
                    phantom: Default::default(),
                }
            }
        }
    }

    impl<T> Drop for FfiBuffer<T> {
        fn drop(&mut self) {
            unsafe {
                Vec::from_raw_parts(self.addr as *mut u8, self.size, self.alloc);
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn __ffi_buffer_address(ptr: *mut c_void) -> *mut c_void {
        let buffer = &*(ptr as *mut FfiBuffer<u8>);
        buffer.addr as _
    }

    #[no_mangle]
    pub unsafe extern "C" fn __ffi_buffer_size(ptr: *mut c_void) -> u32 {
        let buffer = &*(ptr as *mut FfiBuffer<u8>);
        buffer.size as _
    }

    #[no_mangle]
    pub extern "C" fn drop_box_FfiBuffer(_: i64, boxed: i64) {
        panic_abort(move || {
            unsafe { Box::<FfiBuffer<u8>>::from_raw(boxed as *mut _) };
        });
    }

    #[doc=" 'elementAt' method of list types returns a non owning reference"]
    #[no_mangle]
    pub extern "C" fn drop_box_Leak(_: i64, boxed: i64) {
    }

    #[repr(transparent)]
    pub struct FfiIter<T: Send + 'static>(Box<dyn Iterator<Item = T> + Send + 'static>);

    impl<T: Send + 'static> FfiIter<T> {
        pub fn new<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = T>,
            I::IntoIter: Send + 'static,
        {
            Self(Box::new(iter.into_iter()))
        }

        pub fn next(&mut self) -> Option<T> {
            self.0.next()
        }
    }

    #[doc=" Converts a closure into a [`Waker`]."]
    #[doc=""]
    #[doc=" The closure gets called every time the waker is woken."]
    pub fn waker_fn<F: Fn() + Send + Sync + 'static>(f: F) -> Waker {
        let raw = Arc::into_raw(Arc::new(f)) as *const ();
        let vtable = &Helper::<F>::VTABLE;
        unsafe { Waker::from_raw(RawWaker::new(raw, vtable)) }
    }

    struct Helper<F>(F);

    impl<F: Fn() + Send + Sync + 'static> Helper<F> {
        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            Self::clone_waker,
            Self::wake,
            Self::wake_by_ref,
            Self::drop_waker,
        );

        unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
            let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
            core::mem::forget(arc.clone());
            RawWaker::new(ptr, &Self::VTABLE)
        }

        unsafe fn wake(ptr: *const ()) {
            let arc = Arc::from_raw(ptr as *const F);
            (arc)();
        }

        unsafe fn wake_by_ref(ptr: *const ()) {
            let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
            (arc)();
        }

        unsafe fn drop_waker(ptr: *const ()) {
            drop(Arc::from_raw(ptr as *const F));
        }
    }

    fn ffi_waker(_post_cobject: isize, port: i64) -> Waker {
        waker_fn(move || unsafe {
            if cfg!(target_family = "wasm") {
                extern "C" {
                    fn __notifier_callback(idx: i32);
                }
                __notifier_callback(port as _);
            } else {
                let post_cobject: extern "C" fn(i64, *const core::ffi::c_void) =
                    core::mem::transmute(_post_cobject);
                let obj: i32 = 0;
                post_cobject(port, &obj as *const _ as *const _);
            }
        })
    }

    #[repr(transparent)]
    pub struct FfiFuture<T: Send + 'static>(Pin<Box<dyn Future<Output = T> + Send + 'static>>);

    impl<T: Send + 'static> FfiFuture<T> {
        pub fn new(f: impl Future<Output = T> + Send + 'static) -> Self {
            Self(Box::pin(f))
        }

        pub fn poll(&mut self, post_cobject: isize, port: i64) -> Option<T> {
            let waker = ffi_waker(post_cobject, port);
            let mut ctx = Context::from_waker(&waker);
            match Pin::new(&mut self.0).poll(&mut ctx) {
                Poll::Ready(res) => Some(res),
                Poll::Pending => None,
            }
        }
    }

    #[cfg(feature = "test_runner")]
    pub trait Stream {
        type Item;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>>;
    }

    #[cfg(feature = "test_runner")]
    impl<T> Stream for Pin<T>
    where
        T: core::ops::DerefMut + Unpin,
        T::Target: Stream,
    {
        type Item = <T::Target as Stream>::Item;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
            self.get_mut().as_mut().poll_next(cx)
        }
    }

    #[repr(transparent)]
    pub struct FfiStream<T: Send + 'static>(Pin<Box<dyn Stream<Item = T> + Send + 'static>>);

    impl<T: Send + 'static> FfiStream<T> {
        pub fn new(f: impl Stream<Item = T> + Send + 'static) -> Self {
            Self(Box::pin(f))
        }

        pub fn poll(&mut self, post_cobject: isize, port: i64, done: i64) -> Option<T> {
            let waker = ffi_waker(post_cobject, port);
            let mut ctx = Context::from_waker(&waker);
            match Pin::new(&mut self.0).poll_next(&mut ctx) {
                Poll::Ready(Some(res)) => {
                    ffi_waker(post_cobject, port).wake();
                    Some(res)
                }
                Poll::Ready(None) => {
                    ffi_waker(post_cobject, done).wake();
                    None
                }
                Poll::Pending => None,
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn __hello_world() {
        panic_abort(move || {
            hello_world();
        })
    }
    #[no_mangle]
    pub extern "C" fn __async_hello_world() -> i64 {
        panic_abort(move || {
            let tmp0 = async_hello_world();#[allow(unused_assignments)] let mut tmp1 = Default::default();let tmp1_0 = async move { tmp0.await.map_err(|err| err.to_string()) };
            let tmp1_1: FfiFuture<Result<u8>> = FfiFuture::new(tmp1_0);
            tmp1 = Box::into_raw(Box::new(tmp1_1)) as _;
            tmp1
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_image() -> i64 {
        panic_abort(move || {
            let tmp0 = get_image();#[allow(unused_assignments)] let mut tmp1 = Default::default();let tmp0_0 = assert_send_static(tmp0);
            tmp1 = Box::into_raw(Box::new(tmp0_0)) as _;
            tmp1
        })
    }
    #[no_mangle]
    pub extern "C" fn __create(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = create(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_u8_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_u8_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_u16_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_u16_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_u32_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_u32_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_u64_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_u64_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_i8_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_i8_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_i16_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_i16_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_i32_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_i32_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_i64_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_i64_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_f32_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_f32_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __get_f64_counting(tmp1: u64,) -> i64 {
        panic_abort(move || {
            let tmp0 = tmp1 as _;let tmp2 = get_f64_counting(tmp0,);#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn __create_list() -> i64 {
        panic_abort(move || {
            let tmp0 = create_list();#[allow(unused_assignments)] let mut tmp2 = Default::default();let tmp1 = FfiListCustomType::new(tmp0);let tmp1_0 = assert_send_static(tmp1);
            tmp2 = Box::into_raw(Box::new(tmp1_0)) as _;
            tmp2
        })
    }
    #[no_mangle]
    pub extern "C" fn __sum_list(tmp1: i64,) -> u32 {
        panic_abort(move || {
            let tmp2 = unsafe { Box::from_raw(tmp1 as *mut FfiListCustomType) };let tmp0 = tmp2.data;let tmp3 = sum_list(tmp0,);#[allow(unused_assignments)] let mut tmp4 = Default::default();tmp4 = tmp3 as _;
            tmp4
        })
    }
    #[no_mangle]
    pub extern "C" fn __DataTest_get_copy(tmp1: i64,) -> __DataTest_get_copyReturn {
        panic_abort(move || {
            let tmp0 = unsafe { &mut *(tmp1 as *mut DataTest) };let tmp2 = tmp0.get_copy();#[allow(unused_assignments)] let mut tmp3 = Default::default();#[allow(unused_assignments)] let mut tmp4 = Default::default();#[allow(unused_assignments)] let mut tmp5 = Default::default();let tmp2_0 = ManuallyDrop::new(tmp2);
            tmp3 = tmp2_0.as_ptr() as _;
            tmp4 = tmp2_0.len() as _;
            tmp5 = tmp2_0.capacity() as _;
            __DataTest_get_copyReturn {
                ret0: tmp3,ret1: tmp4,ret2: tmp5,
            }
        })
    }
    #[repr(C)]
    pub struct __DataTest_get_copyReturn {
        pub ret0: i64,pub ret1: u64,pub ret2: u64,
    }#[no_mangle]
    pub extern "C" fn __DataTest_get_shmem(tmp1: i64,) -> i64 {
        panic_abort(move || {
            let tmp0 = unsafe { &mut *(tmp1 as *mut DataTest) };let tmp2 = tmp0.get_shmem();#[allow(unused_assignments)] let mut tmp3 = Default::default();let tmp2_0 = assert_send_static(tmp2);
            tmp3 = Box::into_raw(Box::new(tmp2_0)) as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn drop_box_DataTest(_: i64, boxed: i64) {
        panic_abort(move || {
            unsafe { Box::<DataTest>::from_raw(boxed as *mut _) };
        });
    }#[no_mangle]
    pub extern "C" fn __CustomType_get_n(tmp1: i64,) -> i32 {
        panic_abort(move || {
            let tmp0 = unsafe { &mut *(tmp1 as *mut CustomType) };let tmp2 = tmp0.get_n();#[allow(unused_assignments)] let mut tmp3 = Default::default();tmp3 = tmp2 as _;
            tmp3
        })
    }
    #[no_mangle]
    pub extern "C" fn drop_box_CustomType(_: i64, boxed: i64) {
        panic_abort(move || {
            unsafe { Box::<CustomType>::from_raw(boxed as *mut _) };
        });
    }
    #[no_mangle]
    pub extern "C" fn __async_hello_world_future_poll(tmp1: i64,tmp3: i64,tmp5: i64,) -> __async_hello_world_future_pollReturn {
        panic_abort(move || {
            let tmp0 = unsafe { &mut *(tmp1 as *mut FfiFuture<Result<u8>>) };let tmp2 = tmp3 as _;let tmp4 = tmp5 as _;let tmp6 = tmp0.poll(tmp2,tmp4,);#[allow(unused_assignments)] let mut tmp7 = Default::default();#[allow(unused_assignments)] let mut tmp9 = Default::default();#[allow(unused_assignments)] let mut tmp12 = Default::default();#[allow(unused_assignments)] let mut tmp13 = Default::default();#[allow(unused_assignments)] let mut tmp14 = Default::default();#[allow(unused_assignments)] let mut tmp15 = Default::default();if let Some(tmp8) = tmp6 {
                tmp7 = 1;
                match tmp8 {
                    Ok(tmp10) => {
                        tmp9 = 1;
                        tmp15 = tmp10 as _;
                    }
                    Err(tmp11_0) => {
                        tmp9 = 0;
                        let tmp11 = tmp11_0.to_string();
                        let tmp11_0 = ManuallyDrop::new(tmp11);
                        tmp12 = tmp11_0.as_ptr() as _;
                        tmp13 = tmp11_0.len() as _;
                        tmp14 = tmp11_0.capacity() as _;
                    }
                };
            } else {
                tmp7 = 0;
            }
            __async_hello_world_future_pollReturn {
                ret0: tmp7,ret1: tmp9,ret2: tmp12,ret3: tmp13,ret4: tmp14,ret5: tmp15,
            }
        })
    }
    #[repr(C)]
    pub struct __async_hello_world_future_pollReturn {
        pub ret0: u8,pub ret1: u8,pub ret2: i64,pub ret3: u64,pub ret4: u64,pub ret5: u8,
    }
    #[no_mangle]
    pub extern "C" fn __async_hello_world_future_drop(_: i64, boxed: i64) {
        panic_abort(move || {
            unsafe { Box::<FfiFuture<Result<u8>>>::from_raw(boxed as *mut _) };
        });
    }
    pub struct FfiListCustomType {
        data: Vec<CustomType>,
    }

    impl FfiListCustomType {
        fn new(data: Vec<CustomType>) -> Self {
            Self { data }
        }

        fn len(&self) -> u32 {
            self.data.len() as _
        }

        fn element_at(&self, idx: u32) -> Option<&CustomType> {
            self.data.get(idx as usize)
        }
    }
    #[no_mangle]
    pub extern "C" fn __FfiListCustomTypeCreate() -> usize {
        panic_abort(move || unsafe {
            let list = Box::new(FfiListCustomType::new(vec![]));
            Box::into_raw(list) as _
        })
    }

    #[no_mangle]
    pub extern "C" fn drop_box_FfiListCustomType(_: i64, boxed: i64) {
        panic_abort(move || unsafe {
            Box::<FfiListCustomType>::from_raw(boxed as _);
        })
    }

    #[no_mangle]
    pub extern "C" fn __FfiListCustomTypeLen(boxed: usize) -> u32 {
        panic_abort(move || unsafe {
            let list = Box::<FfiListCustomType>::from_raw(boxed as _);
            let result = list.len();
            Box::into_raw(list);
            result as _
        })
    }

    #[no_mangle]
    pub extern "C" fn __FfiListCustomTypeElementAt(boxed: usize, index: u32) -> usize {
        panic_abort(move || unsafe {
            let list = Box::<FfiListCustomType>::from_raw(boxed as _);
            let result = list.element_at(index).unwrap() as *const _;
            Box::into_raw(list);
            result as _
        })
    }
}
