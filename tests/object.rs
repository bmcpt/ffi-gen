use ffi_gen::{compile_pass, compile_pass_no_js};

compile_pass! {
    object,
    r#"
    fn create(value: u32) -> CustomType;
    fn was_dropped() -> bool;
    object CustomType {
        static fn create(value: u32) -> CustomType;
        fn do_something() -> u32;
    }
    "#,
    (
        use std::sync::atomic::{AtomicBool, Ordering};

        static WAS_DROPPED: AtomicBool = AtomicBool::new(false);

        pub fn create(value: u32) -> CustomType {
            CustomType::create(value)
        }

        pub struct CustomType {
            value: u32,
        }

        impl Drop for CustomType {
            fn drop(&mut self) {
                WAS_DROPPED.store(true, Ordering::SeqCst);
            }
        }

        impl CustomType {
            pub fn create(value: u32) -> Self {
                Self { value }
            }

            pub fn do_something(&self) -> u32 {
                self.value
            }
        }

        pub fn was_dropped() -> bool {
            WAS_DROPPED.load(Ordering::SeqCst)
        }
    ),
    (
        let boxed = __CustomType_create(42);
        assert_eq!(__CustomType_do_something(boxed), 42);
        drop_box_CustomType(0 as _, boxed);
        assert!(was_dropped());

        let boxed = __create(42);
        assert_eq!(__CustomType_do_something(boxed), 42);
        drop_box_CustomType(0 as _, boxed);
        assert!(was_dropped());
    ),
    (
        final boxed = CustomType.create(api, 42);
        assert(boxed.doSomething() == 42);
        boxed.drop();
        assert(api.wasDropped());

        final obj = api.create(42);
        assert(obj.doSomething() == 42);
        obj.drop();
        assert(api.wasDropped());
    ),
    (
        const boxed = CustomType.create(api, 42);
        assert.equal(boxed.doSomething(), 42);
        boxed.drop();
        assert.equal(api.wasDropped(), true);

        const obj = api.create(42);
        assert.equal(obj.doSomething(), 42);
        obj.drop();
        assert.equal(api.wasDropped(), true);
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        create(value: number): CustomType;

        wasDropped(): boolean;
    }

    export class CustomType {
        static create(api: Api, value: number): CustomType;

        doSomething(): number;

        drop(): void;
    })

}

compile_pass! {
    iterator,
    r#"fn vec_str() -> Iterator<string>;
    //fn vec_vec_str() -> Iterator<Iterator<string>>;
    "#,
    (
        pub fn vec_str() -> Vec<String> {
            vec!["hello".into(), "world".into()]
        }

        /*pub fn vec_vec_str() -> Vec<Vec<String>> {
            vec![vec!["hello".into()], vec!["world".into()]]
        }*/
    ),
    (
        let iter = __vec_str();
        assert_eq!(__vec_str_iter_next(iter).ret0, 1);
        assert_eq!(__vec_str_iter_next(iter).ret0, 1);
        assert_eq!(__vec_str_iter_next(iter).ret0, 0);
        __vec_str_iter_drop(0, iter);
    ),
    (
        final List<String> res = [];
        for (final s in api.vecStr()) {
            res.add(s);
        }
        assert(res.length == 2);
        assert(res[0] == "hello");
        assert(res[1] == "world");

        /*final res = api.vecVecStr(); //[["hello"], ["world"]]);
        assert(res.length == 2);
        assert(res[0].length == 1);
        assert(res[0][0] == "hello");
        assert(res[1].length == 1);
        assert(res[1][0] == "world");*/
    ),
    (
        const res = [];
        let iter = api.vecStr();
        for (const el of iter) {
            res.push(el);
        }
        assert(res.length == 2);
        assert(res[0] == "hello");
        assert(res[1] == "world");
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        vecStr(): Iterable<string>;
    })
}

compile_pass! {
    nodelay_future,
    "fn create(value: u32) -> Future<u32>;",
    (
        pub async fn create(value: u32) -> u32 {
            value
        }
    ),
    (
        let _fut = __create(42);
        let _f = __create_future_poll;
        __create_future_drop(0, _fut);
    ),
    (
        final fut = api.create(42);
        assert(await fut == 42);
    ),
    (
        const fut = api.create(42);
        assert.equal(await fut, 42);
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        create(value: number): Promise<number>;
    })
}

compile_pass! {
    delayed_future,
    "fn create() -> Future<u64>; fn wake();",
    (
        use core::future::Future;
        use core::pin::Pin;
        use core::task::{Context, Poll, Waker};

        static mut WAKER: Option<Waker> = None;
        static mut WOKEN: bool = false;

        pub struct Delayed;

        impl Future for Delayed {
            type Output = u64;

            fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
                unsafe {
                    if !WOKEN {
                        WAKER = Some(cx.waker().clone());
                        Poll::Pending
                    } else {
                        Poll::Ready(42)
                    }
                }
            }
        }

        pub fn create() -> Delayed {
            Delayed
        }

        pub fn wake() {
            unsafe {
                if let Some(waker) = WAKER.take() {
                    WOKEN = true;
                    waker.wake();
                }
            }
        }
    ),
    (
        let fut = __create();
        let _poll = __create_future_poll;
        __create_future_drop(0, fut);
    ),
    (
        final fut = api.create();
        api.wake();
        assert(await fut == 42);
    ),
    (
        const i = setInterval(() => {
            // do nothing but prevent node process from exiting
        }, 1000);

        const fut = api.create();
        api.wake();
        console.log(fut);
        const res = await fut;
        clearInterval(i);
        console.log(res);
        assert.equal(res, 42);
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        create(): Promise<BigInt>;

        wake(): void;
    })
}

compile_pass! {
    nodelay_stream,
    "fn create(values: &[u32]) -> Stream<u32>;",
    (
        use crate::api::Stream;
        use core::pin::Pin;
        use core::task::{Context, Poll};

        struct TestStream(Vec<u32>);

        impl Stream for TestStream {
            type Item = u32;

            fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Option<Self::Item>> {
                Poll::Ready(self.0.pop())
            }
        }

        pub fn create(values: &[u32]) -> impl Stream<Item = u32> {
            TestStream(values.into_iter().rev().copied().collect())
        }
    ),
    (
        let values = [42, 99];
        let stream = __create(values.as_ptr() as _, values.len() as _);

        extern "C" fn callback(port: i64, _obj: &i32) {
            assert!(port == 0 || port == 1);
        }

        let poll = __create_stream_poll(stream, callback as *const core::ffi::c_void as _, 0, 1);
        assert_eq!(poll.ret0, 1);
        assert_eq!(poll.ret1, 42);
        let poll = __create_stream_poll(stream, callback as *const core::ffi::c_void as _, 0, 1);
        assert_eq!(poll.ret0, 1);
        assert_eq!(poll.ret1, 99);
        let poll = __create_stream_poll(stream, callback as *const core::ffi::c_void as _, 0, 1);
        assert_eq!(poll.ret0, 0);

        __create_stream_drop(0, stream);
    ),
    (
        final stream = api.create([42, 99]);
        var counter = 0;
        await for (final value in stream) {
            assert(counter == 0 && value == 42 || counter == 1 && value == 99);
            counter += 1;
        }
        assert(counter == 2);
    ),
    (
        const i = setTimeout(() => {
            // do nothing but prevent node process from exiting
        }, 1000);

        const stream = api.create([42, 99]);
        let counter = 0;
        for await (const value of stream) {
            assert(counter == 0 && value == 42 || counter == 1 && value == 99);
            counter += 1;
        }
        assert(counter == 2);
        clearInterval(i);
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        create(values: Array<number>): ReadableStream<number>;
    })
}

compile_pass! {
    result_future,
    "fn create(value: u32) -> Future<Result<u32>>;",
    (
        pub async fn create(value: u32) -> Result<u32, &'static str> {
            if value == 0 {
                Err("is zero")
            } else {
                Ok(value)
            }
        }
    ),
    (
        let _fut = __create(42);
        let _f = __create_future_poll;
        let _d = __create_future_drop;
    ),
    (
        final fut = api.create(42);
        assert(await fut == 42);

        var err = false;
        try {
            final fut = api.create(0);
            assert(await fut == 99);
        } catch(ex) {
            assert(ex == "is zero");
            err = true;
        }
        assert(err);
    ),
    (
        const fut = api.create(42);
        assert.equal(await fut, 42);

        let err = false;
        try {
            const fut = api.create(0);
            assert.equal(await fut, 99);
        } catch(ex) {
            assert.equal(ex, "is zero");
            err = true;
        }
        assert.equal(err, true);
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        create(value: number): Promise<number>;
    })
}

compile_pass! {
    future_iterator,
    "fn future_iterator() -> Future<Iterator<string>>;",
    (
        pub async fn future_iterator() -> Vec<String> {
            vec!["hello".to_string(), "world".to_string()]
        }
    ),
    ( ),
    (
        final iter = await api.futureIterator();
        final list = [];
        for (final item in iter) {
            list.add(item);
        }
        assert(list.length == 2);
        assert(list[0] == "hello");
        assert(list[1] == "world");
    ),
    (
        const iter = await api.futureIterator();
        const list = [];
        for (const item of iter) {
            list.push(item);
        }
        assert.equal(list.length, 2);
        assert.equal(list[0], "hello");
        assert.equal(list[1], "world");
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        futureIterator(): Promise<Iterable<string>>;
    })
}

compile_pass_no_js! {
    typed_buffers,
    "
        fn u8(n: usize) -> buffer<u8>;
        fn u16(n: usize) -> buffer<u16>;
        fn u32(n: usize) -> buffer<u32>;
        fn u64(n: usize) -> buffer<u64>;
        fn i8(n: usize) -> buffer<i8>;
        fn i16(n: usize) -> buffer<i16>;
        fn i32(n: usize) -> buffer<i32>;
        fn i64(n: usize) -> buffer<i64>;
        fn f32(n: usize) -> buffer<f32>;
        fn f64(n: usize) -> buffer<f64>;
    ",
    (
        macro_rules! gen_counting_func {
            ($ty:ident) => {
                pub fn $ty(n: usize) -> api::FfiBuffer<$ty> {
                    api::FfiBuffer::new((0..n).map(|n| n as $ty).collect())
                }
            };
        }

        gen_counting_func!(u8);
        gen_counting_func!(u16);
        gen_counting_func!(u32);
        gen_counting_func!(u64);
        gen_counting_func!(i8);
        gen_counting_func!(i16);
        gen_counting_func!(i32);
        gen_counting_func!(i64);
        gen_counting_func!(f32);
        gen_counting_func!(f64);
    ),
    (
        use std::ffi::c_void;
        unsafe {
            type el = u16;
            let ptr = __u16(20) as *mut c_void;
            let addr = __ffi_buffer_address(ptr);
            let size = __ffi_buffer_size(ptr);
            let buf = std::slice::from_raw_parts(addr as *mut el, size as usize / std::mem::size_of::<el>());
            dbg!(buf);
        }
    ),
    (
        final buffers = <dynamic>[
            api.u8(20),
            api.u16(20),
            api.u32(20),
            api.u64(20),
            api.i8(20),
            api.i16(20),
            api.i32(20),
            api.i64(20),
            api.f32(20),
            api.f64(20),
        ];
        final views = buffers.map((b) => b.asTypedList());
        views.forEach((v) {
            assert(v.length == 20, v.toString());
            for (var i=0; i<20; i++) {
                assert(v[i] == i, v.toString());
            }
        });
    )
}

compile_pass_no_js! {
    vec_string,
    "fn ss() -> Vec<string>;",
    (
        pub fn ss() -> Vec<String> {
            vec!["hello".to_string(), "world".to_string()]
        }
    ),
    (),
    (
        final ss = api.ss().map((s) => s.toDartString()).toList();
        assert(ss.length == 2);
        assert(ss[0] == "hello", ss[0]);
        assert(ss[1] == "world", ss[1]);
    )
}

compile_pass_no_js! {
    typed_list,
    "object CustomObject {
        fn to_string() -> string;
    }

    fn create_custom_objects(n: usize) -> Future<Vec<CustomObject>>;
    ",
    (
        pub struct CustomObject {
            n: usize
        }

        impl CustomObject {
            fn new(n: usize) -> Self {
                Self { n }
            }

            fn to_string(&self) -> String {
                format!("CustomObject({})", self.n)
            }
        }

        async fn create_custom_objects(n: usize) -> Vec<CustomObject> {
            return (0..n).map(CustomObject::new).collect()
        }
    ),
    (),
    (
        final objs = await api.createCustomObjects(40);
        final reprs = objs.map((o) => o.toString()).toList();
        assert(reprs[5].toString() == "CustomObject(5)", reprs);
        assert(reprs[34].toString() == "CustomObject(34)", reprs);
    )
}

compile_pass_no_js! {
    read_file,
    "
    fn read_file(path: string) -> Future<buffer<u8>>;
    ",
    (
        async fn read_file(path: String) -> api::FfiBuffer<u8> {
            let data = std::fs::read(path.as_str()).unwrap();
            api::FfiBuffer::new(data)
        }
    ),
    (),
    (
        final path = "../../../example/dart/image.jpg";
        final img1 = await new File(path).readAsBytes();
        final img2 = await api.readFile(path);
        assert(img1.equals(img2.asTypedList()));
    )
}

compile_pass_no_js! {
    enums,
    "
object Vector2 {
    fn x() -> u64;
    fn y() -> u64;
}

object Vector3 {
    fn x() -> u64;
    fn y() -> u64;
    fn z() -> u64;
}

enum Shape {
    Square(Vector2),
    Cube(Vector3),
    None
}

fn get_shape() -> Shape;

fn get_shapes() -> Vec<Shape>;
    ",
    (
        #[derive(Debug, Clone, Copy)]
        pub struct Vector2 {
            x: u64,
            y: u64,
        }

        impl Vector2 {
            pub fn x(&self) -> u64 {
                self.x
            }
            pub fn y(&self) -> u64 {
                self.y
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub struct Vector3 {
            x: u64,
            y: u64,
            z: u64,
        }

        impl Vector3 {
            pub fn x(&self) -> u64 {
                self.x
            }
            pub fn y(&self) -> u64 {
                self.y
            }
            pub fn z(&self) -> u64 {
                self.z
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub enum Shape {
            Square(Vector2),
            Cube(Vector3),
            None,
        }

        fn get_shape() -> Shape {
            Shape::Square(Vector2 { x: 0, y: 5 })
        }

        fn get_shapes() -> Vec<Shape> {
            use Shape::*;
            vec![
                Square(Vector2 { x: 5, y: 3 }),
                None,
                Cube(Vector3 { x: 4, y: 0, z: 1 }),
                Square(Vector2 { x: 5, y: 3 }),
                None,
                None,
                Square(Vector2 { x: 5, y: 3 }),
                Cube(Vector3 { x: 4, y: 0, z: 1 }),
            ]
        }
    ),
    (),
    (
        final shapes = api.getShapes();
        assert(shapes.length == 8);

        assert(shapes.elementAt(0).tag == ShapeTag.Square);
        assert(shapes.elementAt(0).inner.runtimeType == Vector2, shapes.elementAt(0).inner.runtimeType);
        var s2 = shapes.elementAt(0).inner as Vector2;
        assert(s2.x() == 5);
        assert(s2.y() == 3);

        assert(shapes.elementAt(1).tag == ShapeTag.None);
        assert(shapes.elementAt(1).inner == null, shapes.elementAt(1).inner);

        assert(shapes.elementAt(2).tag == ShapeTag.Cube);
        assert(shapes.elementAt(2).inner.runtimeType == Vector3, shapes.elementAt(2).inner.runtimeType);
        var s3 = shapes.elementAt(2).inner as Vector3;
        assert(s3.x() == 4);
        assert(s3.y() == 0);
        assert(s3.z() == 1);
    )
}

compile_pass_no_js! {
    future_vec_string,
    "\
    fn strings() -> Future<Vec<string>>;
    ",
    (
        async fn strings() -> Vec<String> {
            vec![
                "a",
                "b",
                "c",
            ].iter().map(|s| s.to_string()).collect()
        }
    ),
    (),
    (
        final strings = await api.strings();
        print(strings);
        assert(strings.length == 3);
        assert(strings[0].toDartString() == "a");
        assert(strings[1].toDartString() == "b");
        assert(strings[2].toDartString() == "c");
    )
}

compile_pass_no_js! {
    ffilist_modification,
    "
    fn create_list(n: usize) -> Future<Vec<CustomType>>;

    fn sum_list(l: Vec<CustomType>) -> u32;

    object CustomType {
        fn get_n() -> usize;
    }

    fn create_custom_type(n: usize) -> CustomType;
    ",
    (
        #[derive(Debug)]
        pub struct CustomType {
            n: usize,
        }

        impl CustomType {
            fn get_n(&self) -> usize {
                self.n
            }
        }

        pub fn create_custom_type(n: usize) -> CustomType {
            CustomType { n }
        }

        async fn create_list(n: usize) -> Vec<CustomType> {
            (0..n)
                .map(|n| CustomType { n })
                .collect()
        }

        fn sum_list(l: &[CustomType]) -> usize {
            l.iter().map(|e| e.n).sum()
        }
    ),
    (),
    (
        final list = await api.createList(5);
        final el = list.remove(2);
        list.insert(0, el);
        list.add(api.createCustomType(30));

        assert(list.length == 6);
        final expected = [2, 0, 1, 3, 4, 30];
        for (int i=0; i<6; i++) {
            assert(expected[i] == list[i].getN());
        }
        assert(api.sumList(list) == 40);
    )
}
