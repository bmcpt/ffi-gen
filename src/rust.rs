use crate::export::Instr;
use crate::parser::Enum;
use crate::{
    Abi, AbiFunction, AbiFuture, AbiIter, AbiObject, AbiStream, AbiType, FunctionType, Interface,
    NumType, Return, Var,
};
use genco::prelude::*;

pub struct RustGenerator {
    abi: Abi,
}

impl RustGenerator {
    pub fn new(abi: Abi) -> Self {
        Self { abi }
    }

    pub fn generate(&self, iface: Interface) -> rust::Tokens {
        let wasm_bindgen: rust::Tokens = if cfg!(feature = "wasm-bindgen") {
            quote! {
                // Workaround for combined use with `wasm-bindgen`, so we don't have to
                // patch the `importObject` while loading the WASM module.
                #[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen(js_namespace = window, js_name = __notifier_callback))]
            }
        } else {
            quote!()
        };
        quote! {
        #[allow(unused)]
        pub mod api {
            #![allow(clippy::all)]
            use core::future::Future;
            use core::mem::ManuallyDrop;
            use core::pin::Pin;
            use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
            use std::sync::Arc;
            use std::ffi::c_void;
            use super::*;

            type FfiString = String;

            /// Try to execute some function, catching any panics and aborting to make sure Rust
            /// doesn't unwind across the FFI boundary.
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

            #[derive(Debug)]
            pub struct FfiBuffer<T> {
                pub addr: usize,
                pub size: usize,
                pub alloc: usize,
                pub phantom: std::marker::PhantomData<T>
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
            pub unsafe extern "C" fn __ffi_buffer_size(ptr: *mut c_void) -> u64 {
                let buffer = &*(ptr as *mut FfiBuffer<u8>);
                buffer.size as _
            }

            #[repr(C)]
            pub struct _FfiStringParts {
                ptr: i64,
                len: u64,
                capacity: u64,
            }

            #[no_mangle]
            pub unsafe extern "C" fn __ffi_string_into_parts(ptr: *mut c_void) -> _FfiStringParts {
                let s = &*(ptr as *mut FfiString);
                let obj = ManuallyDrop::new(s);
                _FfiStringParts {
                    ptr: obj.as_ptr() as _,
                    len: obj.len() as _,
                    capacity: obj.capacity() as _,
                }
            }

            #[no_mangle]
            pub extern "C" fn drop_box_FfiBuffer(_: i64, boxed: i64) {
                panic_abort(move || {
                    unsafe { Box::<FfiBuffer<u8>>::from_raw(boxed as *mut _) };
                });
            }

            #[no_mangle]
            pub extern "C" fn drop_box_Leak(_: i64, boxed: i64) {
            }

            #[repr(C)]
            pub struct EnumWrapper {
                tag: u32,
                inner: *mut c_void,
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

            /// Converts a closure into a [`Waker`].
            ///
            /// The closure gets called every time the waker is woken.
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
                        #(wasm_bindgen)
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

            #(for func in iface.functions() => #(self.generate_function(&func)))
            #(for obj in iface.objects() => #(self.generate_object(&obj)))
            #(for iter in iface.iterators() => #(self.generate_iterator(&iter)))
            #(for fut in iface.futures() => #(self.generate_future(&fut)))
            #(for stream in iface.streams() => #(self.generate_stream(&stream)))
            #(for ty in iface.listed_types() => #(self.generate_list_type_methods(ty.as_str())))
            #(for e in iface.enums.iter() => #(self.generate_enum_helpers(e)))
        }
        }
    }

    fn generate_enum_helpers(&self, e: &Enum) -> rust::Tokens {
        let destructure_function_name = format!("destructure_enum_{}", e.ident);
        let drop_function_name = format!("drop_box_{}", e.ident);
        let mut entry_index = -1;
        quote!(
            #[no_mangle]
            pub unsafe fn #(&destructure_function_name)(ptr: *mut c_void) -> EnumWrapper {
                let e = &*(ptr as *mut #(&e.ident)).clone();
                let (tag, inner) = match *e {
                    #(for sub in &e.entries => #(&e.ident)::#(&sub.name)
                        #(if sub.inner.is_some() { (inner) }) =>
                            (
                                #({ entry_index += 1; entry_index }),
                                #(if sub.inner.is_some() { Box::into_raw(Box::new(inner)) as _ } else { 0 as _ }),
                            ),
                    )
                };
                EnumWrapper {
                    tag,
                    inner,
                }
            }

            #[no_mangle]
            pub extern "C" fn #(&drop_function_name)(_: i64, boxed: i64) {
                panic_abort(move || {
                    unsafe { Box::<#(&e.ident)>::from_raw(boxed as *mut _) };
                });
            }

        )
    }

    fn generate_list_type_methods(&self, ty: &str) -> rust::Tokens {
        let name_s = format!("FfiList{}", ty);
        let name = name_s.as_str();
        quote!(
            #[no_mangle]
            pub extern "C" fn #(format!("__{}Create", name))() -> usize {
                panic_abort(move || unsafe {
                    let list = Box::new(Vec::<#ty>::new());
                    Box::into_raw(list) as _
                })
            }

            #[no_mangle]
            pub extern "C" fn #(format!("drop_box_{}", name))(_: i64, boxed: i64) {
                panic_abort(move || unsafe {
                    Box::<Vec<#ty>>::from_raw(boxed as _);
                })
            }

            #[no_mangle]
            pub extern "C" fn #(format!("__{}Len", name))(boxed: usize) -> u32 {
                panic_abort(move || unsafe {
                    let list = Box::<Vec<#ty>>::from_raw(boxed as _);
                    let result = list.len() as u32;
                    Box::into_raw(list);
                    result as _
                })
            }

            #[no_mangle]
            pub extern "C" fn #(format!("__{}ElementAt", name))(boxed: usize, index: u32) -> usize {
                panic_abort(move || unsafe {
                    let list = Box::<Vec<#ty>>::from_raw(boxed as _);
                    let result = list.get(index as usize).unwrap() as *const _;
                    Box::into_raw(list);
                    result as _
                })
            }
        )
    }

    fn generate_function(&self, func: &AbiFunction) -> rust::Tokens {
        let ffi = self.abi.export(func);
        let args = quote!(#(for var in &ffi.ffi_args => #(self.var(var)): #(self.ty(&var.ty)),));
        let ret = match &ffi.ffi_ret {
            Return::Void => quote!(),
            Return::Num(var) => quote!(-> #(self.ty(&var.ty))),
            Return::Struct(_, name) => quote!(-> #name),
        };
        let return_ = match &ffi.ffi_ret {
            Return::Void => quote!(),
            Return::Num(var) => self.var(var),
            Return::Struct(vars, name) => quote! {
                #name {
                    #(for (i, var) in vars.iter().enumerate() => #(format!("ret{}", i)): #(self.var(var)),)
                }
            },
        };
        let return_struct = if let Return::Struct(_, _) = &ffi.ffi_ret {
            self.generate_return_struct(func)
        } else {
            quote!()
        };
        quote! {
            #[no_mangle]
            pub extern "C" fn #(&ffi.symbol)(#args) #ret {
                panic_abort(move || {
                    #(for instr in &ffi.instr => #(self.instr(instr)))
                    #return_
                })
            }
            #return_struct
        }
    }

    fn generate_object(&self, obj: &AbiObject) -> rust::Tokens {
        let destructor_name = format!("drop_box_{}", &obj.name);
        let destructor_type = quote!(#(&obj.name));
        quote! {
            #(for method in &obj.methods => #(self.generate_function(method)))
            #(self.generate_destructor(&destructor_name, destructor_type))
        }
    }

    fn generate_destructor(&self, name: &str, ty: rust::Tokens) -> rust::Tokens {
        // make destructor compatible with dart by adding an unused `isolate_callback_data` as
        // the first argument.
        quote! {
            #[no_mangle]
            pub extern "C" fn #name(_: #(self.ffi_num_type(self.abi.iptr())), boxed: #(self.ffi_num_type(self.abi.iptr()))) {
                panic_abort(move || {
                    unsafe { Box::<#ty>::from_raw(boxed as *mut _) };
                });
            }
        }
    }

    fn generate_iterator(&self, iter: &AbiIter) -> rust::Tokens {
        let destructor_name = format!("{}_iter_drop", &iter.symbol);
        let destructor_type = quote!(FfiIter<#(self.ty(&iter.ty))>);
        quote! {
            #(self.generate_function(&iter.next()))
            #(self.generate_destructor(&destructor_name, destructor_type))
        }
    }

    fn generate_future(&self, fut: &AbiFuture) -> rust::Tokens {
        let destructor_name = format!("{}_future_drop", &fut.symbol);
        let destructor_type = quote!(FfiFuture<#(self.ty(&fut.ty))>);
        quote! {
            #(self.generate_function(&fut.poll()))
            #(self.generate_destructor(&destructor_name, destructor_type))
        }
    }

    fn generate_stream(&self, stream: &AbiStream) -> rust::Tokens {
        let destructor_name = format!("{}_stream_drop", &stream.symbol);
        let destructor_type = quote!(FfiStream<#(self.ty(&stream.ty))>);
        quote! {
            #(self.generate_function(&stream.poll()))
            #(self.generate_destructor(&destructor_name, destructor_type))
        }
    }

    fn generate_return_struct(&self, func: &AbiFunction) -> rust::Tokens {
        let ffi = self.abi.export(func);
        if let Return::Struct(vars, name) = &ffi.ffi_ret {
            quote! {
                #[repr(C)]
                pub struct #name {
                    #(for (i, var) in vars.iter().enumerate() => #(format!("pub ret{}", i)): #(self.ty(&var.ty)),)
                }
            }
        } else {
            quote!()
        }
    }

    fn instr(&self, instr: &Instr) -> rust::Tokens {
        match instr {
            Instr::LiftNum(in_, out) | Instr::LiftIsize(in_, out) | Instr::LiftUsize(in_, out) => {
                quote!(let #(self.var(out)) = #(self.var(in_)) as _;)
            }
            Instr::LiftNumAsU32Tuple(in_low, in_high, out, num_type) => {
                let ty = self.num_type(*num_type);
                quote! {
                   let #(self.var(out)) = #(ty.clone())::from(#(self.var(in_low))) | (#ty::from(#(self.var(in_high))) << 32);
                }
            }
            Instr::LowerNumAsU32Tuple(r#in, low, high, _num_type) => {
                quote! {
                    #(self.var(low)) = #(self.var(r#in)) as u32;
                    #(self.var(high)) = (#(self.var(r#in)) >> 32) as u32;
                }
            }
            Instr::LowerNum(in_, out)
            | Instr::LowerIsize(in_, out)
            | Instr::LowerUsize(in_, out) => quote!(#(self.var(out)) = #(self.var(in_)) as _;),
            Instr::LiftBool(in_, out) => quote!(let #(self.var(out)) = #(self.var(in_)) > 0;),
            Instr::LowerBool(in_, out) => {
                quote!(#(self.var(out)) = if #(self.var(in_)) { 1 } else { 0 };)
            }
            Instr::LiftStr(ptr, len, out) => quote! {
                let #(self.var(out))_0: &[u8] =
                    unsafe { core::slice::from_raw_parts(#(self.var(ptr)) as _, #(self.var(len)) as _) };
                let #(self.var(out)): &str = unsafe { std::str::from_utf8_unchecked(#(self.var(out))_0) };
            },
            Instr::LowerStr(in_, ptr, len) => quote! {
                #(self.var(ptr)) = #(self.var(in_)).as_ptr() as _;
                #(self.var(len)) = #(self.var(in_)).len() as _;
            },
            Instr::LiftString(ptr, len, cap, out) => quote! {
                let #(self.var(out)) = unsafe {
                    String::from_raw_parts(
                        #(self.var(ptr)) as _,
                        #(self.var(len)) as _,
                        #(self.var(cap)) as _,
                    )
                };
            },
            Instr::LowerString(in_, ptr, len, cap) | Instr::LowerVec(in_, ptr, len, cap, _) => {
                quote! {
                    let #(self.var(in_))_0 = ManuallyDrop::new(#(self.var(in_)));
                    #(self.var(ptr)) = #(self.var(in_))_0.as_ptr() as _;
                    #(self.var(len)) = #(self.var(in_))_0.len() as _;
                    #(self.var(cap)) = #(self.var(in_))_0.capacity() as _;
                }
            }
            Instr::LiftSlice(ptr, len, out, ty) => quote! {
                let #(self.var(out)): &[#(self.num_type(*ty))] =
                    unsafe { core::slice::from_raw_parts(#(self.var(ptr)) as _, #(self.var(len)) as _) };
            },
            Instr::LowerSlice(in_, ptr, len, _ty) => quote! {
                #(self.var(ptr)) = #(self.var(in_)).as_ptr() as _;
                #(self.var(len)) = #(self.var(in_)).len() as _;
            },
            Instr::LiftVec(ptr, len, cap, out, ty) => quote! {
                let #(self.var(out)) = unsafe {
                    Vec::<#(self.num_type(*ty))>::from_raw_parts(
                        #(self.var(ptr)) as _,
                        #(self.var(len)) as _,
                        #(self.var(cap)) as _,
                    )
                };
            },
            Instr::LiftRefObject(in_, out, object) => quote! {
                let #(self.var(out)) = unsafe { &mut *(#(self.var(in_)) as *mut #object) };
            },
            Instr::LowerRefObject(in_, out) => quote! {
                #(self.var(out)) = #(self.var(in_)) as *const _ as _;
            },
            Instr::LiftObject(in_, out, object) => quote! {
                let #(self.var(out)) = unsafe { Box::from_raw(#(self.var(in_)) as *mut #object) };
            },
            Instr::LowerObject(in_, out) => quote! {
                let #(self.var(in_))_0 = assert_send_static(#(self.var(in_)));
                #(self.var(out)) = Box::into_raw(Box::new(#(self.var(in_))_0)) as _;
            },
            Instr::LiftRefIter(in_, out, ty) => quote! {
                let #(self.var(out)) = unsafe { &mut *(#(self.var(in_)) as *mut FfiIter<#(self.ty(ty))>) };
            },
            Instr::LowerRefIter(in_, out, _ty) => quote! {
                #(self.var(out)) = #(self.var(in_)) as *const _ as _;
            },
            Instr::LiftIter(in_, out, ty) => quote! {
                let #(self.var(out)) = unsafe { Box::from_raw(#(self.var(in_)) as *mut FfiIter<#(self.ty(ty))>) };
            },
            Instr::LowerIter(in_, out, ty) => {
                let iter = if let AbiType::Result(_) = ty {
                    quote!(#(self.var(in_)).map_err(|err| err.to_string()))
                } else {
                    quote!(#(self.var(in_)))
                };
                quote! {
                    let #(self.var(out))_0 = #iter;
                    let #(self.var(out))_1: FfiIter<#(self.ty(ty))> = FfiIter::new(#(self.var(out))_0);
                    #(self.var(out)) = Box::into_raw(Box::new(#(self.var(out))_1)) as _;
                }
            }
            Instr::LiftRefFuture(in_, out, ty) => quote! {
                let #(self.var(out)) = unsafe { &mut *(#(self.var(in_)) as *mut FfiFuture<#(self.ty(ty))>) };
            },
            Instr::LowerRefFuture(in_, out, _ty) => quote! {
                #(self.var(out)) = #(self.var(in_)) as *const _ as _;
            },
            Instr::LiftFuture(in_, out, ty) => quote! {
                let #(self.var(out)) = unsafe { Box::from_raw(#(self.var(in_)) as *mut FfiFuture<#(self.ty(ty))>) };
            },
            Instr::LowerFuture(in_, out, ty) => {
                let future = if let AbiType::Result(_) = ty {
                    quote!(async move { #(self.var(in_)).await.map_err(|err| err.to_string()) })
                } else {
                    quote!(#(self.var(in_)))
                };
                quote! {
                    let #(self.var(out))_0 = #future;
                    let #(self.var(out))_1: FfiFuture<#(self.ty(ty))> = FfiFuture::new(#(self.var(out))_0);
                    #(self.var(out)) = Box::into_raw(Box::new(#(self.var(out))_1)) as _;
                }
            }
            Instr::LiftRefStream(in_, out, ty) => quote! {
                let #(self.var(out)) = unsafe { &mut *(#(self.var(in_)) as *mut FfiStream<#(self.ty(ty))>) };
            },
            Instr::LowerRefStream(in_, out, _ty) => quote! {
                #(self.var(out)) = #(self.var(in_)) as *const _ as _;
            },
            Instr::LiftStream(in_, out, ty) => quote! {
                let #(self.var(out)) = unsafe { Box::from_raw(#(self.var(in_)) as *mut FfiStream<#(self.ty(ty))>) };
            },
            Instr::LowerStream(in_, out, ty) => {
                let map_err = if let AbiType::Result(_) = ty {
                    quote!(.map_err(|err| err.to_string()))
                } else {
                    quote!()
                };
                quote! {
                    let #(self.var(out))_0: FfiStream<#(self.ty(ty))> = FfiStream::new(#(self.var(in_))#map_err);
                    #(self.var(out)) = Box::into_raw(Box::new(#(self.var(out))_0)) as _;
                }
            }
            Instr::LiftOption(var, out, inner, inner_instr) => quote! {
                let #(self.var(out)) = if #(self.var(var)) == 0 {
                    None
                } else {
                    #(for instr in inner_instr => #(self.instr(instr)))
                    Some(#(self.var(inner)))
                };
            },
            Instr::LowerOption(in_, var, some, some_instr) => quote! {
                if let Some(#(self.var(some))) = #(self.var(in_)) {
                    #(self.var(var)) = 1;
                    #(for instr in some_instr => #(self.instr(instr)))
                } else {
                    #(self.var(var)) = 0;
                }
            },
            Instr::LowerResult(in_, var, ok, ok_instr, err, err_instr) => quote! {
                match #(self.var(in_)) {
                    Ok(#(self.var(ok))) => {
                        #(self.var(var)) = 1;
                        #(for instr in ok_instr => #(self.instr(instr)))
                    }
                    Err(#(self.var(err))_0) => {
                        #(self.var(var)) = 0;
                        let #(self.var(err)) = #(self.var(err))_0.to_string();
                        #(for instr in err_instr => #(self.instr(instr)))
                    }
                };
            },
            Instr::LiftTuple(vars, out) => quote! {
                let #(self.var(out)) = (#(for var in vars => #(self.var(var)),));
            },
            Instr::LowerTuple(ret, vars) => quote! {
                #(for (i, var) in vars.iter().enumerate() => let #(self.var(var)) = #(self.var(ret)).#i;)
            },
            Instr::CallAbi(ty, self_, name, ret, args) => {
                let invoke = match ty {
                    FunctionType::Constructor(object) => {
                        quote!(#object::#name)
                    }
                    FunctionType::Method(_)
                    | FunctionType::NextIter(_, _)
                    | FunctionType::PollFuture(_, _)
                    | FunctionType::PollStream(_, _) => {
                        quote!(#(self.var(self_.as_ref().unwrap())).#name)
                    }
                    FunctionType::Function => {
                        quote!(#name)
                    }
                };
                let args = quote!(#(for arg in args => #(self.var(arg)),));
                if let Some(ret) = ret {
                    quote!(let #(self.var(ret)) = #invoke(#args);)
                } else {
                    quote!(#invoke(#args);)
                }
            }
            Instr::DefineRets(vars) => quote! {
                #(for var in vars => #[allow(unused_assignments)] let mut #(self.var(var)) = Default::default();)
            },
            Instr::AssertType(var, ty) => {
                quote!(let #(self.var(var))_type_test: &#ty = &#(self.var(var));)
            }
        }
    }

    fn var(&self, var: &Var) -> rust::Tokens {
        quote!(#(format!("tmp{}", var.binding)))
    }

    fn ty(&self, ty: &AbiType) -> rust::Tokens {
        match ty {
            AbiType::Num(num) => self.ffi_num_type(*num),
            AbiType::Isize => quote!(isize),
            AbiType::Usize => quote!(usize),
            AbiType::Bool => quote!(bool),
            AbiType::RefStr => quote!(&str),
            AbiType::String => quote!(String),
            AbiType::RefSlice(ty) => quote!(&[#(self.num_type(*ty))]),
            AbiType::Vec(ty) => quote!(Vec<#(self.num_type(*ty))>),
            AbiType::Option(ty) => quote!(Option<#(self.ty(ty))>),
            AbiType::Result(ty) => quote!(Result<#(self.ty(ty))>),
            AbiType::Object(ident) => quote!(#ident),
            AbiType::RefObject(ident) => quote!(&#ident),
            AbiType::Tuple(ty) => quote!((#(for ty in ty => #(self.ty(ty)),))),
            AbiType::RefIter(ty) => quote!(&Vec<#(self.ty(ty))>),
            AbiType::Iter(ty) => quote!(Vec<#(self.ty(ty))>),
            AbiType::RefFuture(ty) => quote!(&impl Future<Output = #(self.ty(ty))>),
            AbiType::Future(ty) => quote!(impl Future<Output = #(self.ty(ty))>),
            AbiType::RefStream(ty) => quote!(&impl Stream<Item = #(self.ty(ty))>),
            AbiType::Stream(ty) => quote!(impl Stream<Item = #(self.ty(ty))>),
            AbiType::Buffer(ty) => quote!(FfiBuffer<#(self.num_type(*ty))>),
            AbiType::List(ty) => quote!(#(format!("Vec<{}>", ty))),
            AbiType::RefEnum(ty) => quote!(#(format!("{}_Wrapper", ty))),
        }
    }

    fn num_type(&self, ty: NumType) -> rust::Tokens {
        match ty {
            NumType::U8 => quote!(u8),
            NumType::U16 => quote!(u16),
            NumType::U32 => quote!(u32),
            NumType::U64 => quote!(u64),
            NumType::I8 => quote!(i8),
            NumType::I16 => quote!(i16),
            NumType::I32 => quote!(i32),
            NumType::I64 => quote!(i64),
            NumType::F32 => quote!(f32),
            NumType::F64 => quote!(f64),
        }
    }

    fn ffi_num_type(&self, ty: NumType) -> rust::Tokens {
        let is_wasm = matches!(self.abi, Abi::Wasm32 | Abi::Wasm64);
        match ty {
            NumType::U8 if !is_wasm => quote!(u8),
            NumType::U16 if !is_wasm => quote!(u16),
            NumType::U8 | NumType::U16 | NumType::U32 => quote!(u32),
            NumType::U64 => quote!(u64),
            NumType::I8 if !is_wasm => quote!(i8),
            NumType::I16 if !is_wasm => quote!(i16),
            NumType::I8 | NumType::I16 | NumType::I32 => quote!(i32),
            NumType::I64 => quote!(i64),
            NumType::F32 => quote!(f32),
            NumType::F64 => quote!(f64),
        }
    }
}

#[cfg(feature = "test_runner")]
#[doc(hidden)]
pub mod test_runner {
    use super::*;
    use anyhow::Result;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use trybuild::TestCases;

    pub fn compile_pass(iface: &str, api: rust::Tokens, test: rust::Tokens) -> Result<()> {
        let iface = Interface::parse(iface)?;
        let gen = RustGenerator::new(Abi::native());
        let gen_tokens = gen.generate(iface);
        let tokens = quote! {
            #gen_tokens
            #api
            fn main() {
                use api::*;
                #test
            }
        };
        let res = tokens.to_file_string()?;
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "#![feature(vec_into_raw_parts)]")?;
        writeln!(tmp, "#![feature(once_cell)]")?;
        tmp.write_all(res.as_bytes())?;
        //println!("{}", res);
        let test = TestCases::new();
        test.pass(tmp.as_ref());
        Ok(())
    }
}
