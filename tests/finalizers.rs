use ffi_gen::{compile_pass, compile_pass_no_js};

compile_pass! {
    finalizers,
    r#"fn make_box() -> CustomType;
    fn was_dropped() -> bool;
    object CustomType {}
    "#,
    (
        use std::sync::atomic::{AtomicBool, Ordering};

        static WAS_DROPPED: AtomicBool = AtomicBool::new(false);

        pub struct CustomType;

        impl Drop for CustomType {
            fn drop(&mut self) {
                WAS_DROPPED.store(true, Ordering::SeqCst);
            }
        }

        pub fn make_box() -> CustomType {
            CustomType
        }

        pub fn was_dropped() -> bool {
            WAS_DROPPED.load(Ordering::SeqCst)
        }
    ),
    (
        let boxed = __make_box();
        drop_box_CustomType(0, boxed);
        assert!(was_dropped());
    ),
    (
        final f = () {
            final boxed = api.makeBox();
        };
        f();
        //boxed.drop();
        //assert(api.wasDropped());
        final largeList = [];
        while (!api.wasDropped()) {
            //sleep(Duration(milliseconds: 10));
            largeList.add(99);
        }
    ),
    (
        const f = () => {
            let boxed = api.makeBox();
        };
        f();
        const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
        //boxed.drop();
        //assert.equal(api.was_dropped(), true);
        while (!api.wasDropped()) {
            await delay(1000);
            global.gc();
        }
    ),
    (
    export class Api {
        constructor();

        fetch(url, imports): Promise<void>;

        makeBox(): CustomType;

        wasDropped(): boolean;
    }

    export class CustomType {

        drop(): void;
    }
    )

}

compile_pass_no_js! {
    finalizers_vec_object,
    "
    object Obj {}
    fn f() -> Vec<Obj>;
    fn o() -> Obj;
    fn has_been_dropped(idx: usize) -> bool;
    ",
    (
        use std::sync::Mutex;
        use std::lazy::SyncLazy;

        const COUNT: usize = 5;
        static DROP_FLAGS: SyncLazy<Mutex<Vec<bool>>> = SyncLazy::new(|| Mutex::new(vec![false; COUNT]));

        #[derive(Debug, Clone)]
        pub struct Obj {
            idx: usize,
        }

        impl Drop for Obj {
            fn drop(&mut self) {
                *(DROP_FLAGS.lock().unwrap().get_mut(self.idx).unwrap()) = true;
            }
        }

        pub fn o() -> Obj {
            Obj { idx: 0 }
        }

        pub fn f() -> Vec<Obj> {
            let mut flags = DROP_FLAGS.lock().unwrap();
            flags.clear();
            let mut result = vec![];
            (0..COUNT).for_each(|idx| {
                result.push(Obj { idx });
                flags.push(false);
            });
            result
        }

        fn has_been_dropped(idx: usize) -> bool {
            DROP_FLAGS.lock().unwrap().get(idx).unwrap().to_owned()
        }
    ),
    (
    ),
    (
        FfiListObj? list;
        Obj? el;
        final gc = () {
            final largeList = [];
            for (int i=0; i<20000000; i++) {
                largeList.add(i);
            }
        };
        const count = 5;
        list = api.f();
        for (int i=0; i<count; i++) {
            assert(api.hasBeenDropped(i) == false);
        }
        el = list.elementAt(count ~/ 3);
        gc();

        for (int i=0; i<count; i++) {
            assert(api.hasBeenDropped(i) == false);
        }
        // lose list reference
        list = null;
        gc();

        // list shouldn't be gc'd since el is keeping it alive
        for (int i=0; i<count; i++) {
            assert(api.hasBeenDropped(i) == false);
        }
        // lose the reference to list element
        el = null;
        gc();

        // now it should be gone
        for (int i=0; i<count; i++) {
            assert(api.hasBeenDropped(i) == true, i.toString());
        }
    )
}