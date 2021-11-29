use crate::import::{Import, Instr};
use crate::{Abi, AbiFunction, AbiObject, AbiType, FunctionType, Interface, NumType};
use genco::prelude::*;
use genco::tokens::static_literal;

pub struct DartGenerator {
    abi: Abi,
    cdylib_name: String,
}

impl DartGenerator {
    pub fn new(cdylib_name: String) -> Self {
        Self {
            abi: Abi::native(),
            cdylib_name,
        }
    }

    pub fn generate(&self, iface: Interface) -> dart::Tokens {
        quote! {
            #(static_literal("//")) AUTO GENERATED FILE, DO NOT EDIT.
            #(static_literal("//"))
            #(static_literal("//")) Generated by "ffi-gen".

            import "dart:convert";
            import "dart:ffi" as ffi;
            import "dart:io" show Platform;
            import "dart:typed_data";

            ffi.Pointer<ffi.Void> _registerFinalizer(Box boxed) {
                final dart = ffi.DynamicLibrary.executable();
                final registerPtr = dart.lookup<ffi.NativeFunction<ffi.Pointer<ffi.Void> Function(
                    ffi.Handle, ffi.Pointer<ffi.Void>, ffi.IntPtr, ffi.Pointer<ffi.Void>)>>("Dart_NewFinalizableHandle");
                final register = registerPtr
                    .asFunction<ffi.Pointer<ffi.Void> Function(
                        Object, ffi.Pointer<ffi.Void>, int, ffi.Pointer<ffi.Void>)>();
                return register(boxed, boxed._ptr, 42, boxed._dropPtr.cast());
            }

            void _unregisterFinalizer(Box boxed) {
                final dart = ffi.DynamicLibrary.executable();
                final unregisterPtr = dart.lookup<ffi.NativeFunction<ffi.Void Function(
                    ffi.Pointer<ffi.Void>, ffi.Handle)>>("Dart_DeleteFinalizableHandle");
                final unregister = unregisterPtr
                    .asFunction<void Function(ffi.Pointer<ffi.Void>, Box)>();
                unregister(boxed._finalizer, boxed);
            }

            class Box {
                final Api _api;
                final ffi.Pointer<ffi.Void> _ptr;
                final String _drop_symbol;
                bool _dropped;
                bool _moved;
                ffi.Pointer<ffi.Void> _finalizer = ffi.Pointer.fromAddress(0);

                Box(this._api, this._ptr, this._drop_symbol) : _dropped = false, _moved = false;

                late final _dropPtr = this._api._lookup<
                    ffi.NativeFunction<
                        ffi.Void Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>)>>(this._drop_symbol);

                late final _drop = _dropPtr.asFunction<
                    void Function(ffi.Pointer<ffi.Void>, ffi.Pointer<ffi.Void>)>();

                int borrow() {
                    if (this._dropped) {
                        throw new StateError("use after free");
                    }
                    if (this._moved) {
                        throw new StateError("use after move");
                    }
                    return this._ptr.address;
                }

                int move() {
                    if (this._dropped) {
                        throw new StateError("use after free");
                    }
                    if (this._moved) {
                        throw new StateError("can't move value twice");
                    }
                    this._moved = true;
                    _unregisterFinalizer(this);
                    return this._ptr.address;
                }

                void drop() {
                    if (this._dropped) {
                        throw new StateError("double free");
                    }
                    if (this._moved) {
                        throw new StateError("can't drop moved value");
                    }
                    this._dropped = true;
                    _unregisterFinalizer(this);
                    this._drop(ffi.Pointer.fromAddress(0), this._ptr);
                }
            }

            class Api {
                #(static_literal("///")) Holds the symbol lookup function.
                final ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
                    _lookup;

                #(static_literal("///")) The symbols are looked up in [dynamicLibrary].
                Api(ffi.DynamicLibrary dynamicLibrary)
                    : _lookup = dynamicLibrary.lookup;

                #(static_literal("///")) The symbols are looked up with [lookup].
                Api.fromLookup(
                    ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
                        lookup)
                    : _lookup = lookup;

                #(static_literal("///")) The library is loaded from the executable.
                factory Api.loadStatic() {
                    return Api(ffi.DynamicLibrary.executable());
                }

                #(static_literal("///")) The library is dynamically loaded.
                factory Api.loadDynamic(String name) {
                    return Api(ffi.DynamicLibrary.open(name));
                }

                #(static_literal("///")) The library is loaded based on platform conventions.
                factory Api.load() {
                    String? name;
                    if (Platform.isLinux) name = #_(#("lib")#(&self.cdylib_name)#(".so"));
                    if (Platform.isAndroid) name = #_(#("lib")#(&self.cdylib_name)#(".so"));
                    if (Platform.isMacOS) name = #_(#("lib")#(&self.cdylib_name)#(".dylib"));
                    if (Platform.isIOS) name = #_("");
                    if (Platform.isWindows) #_(#(&self.cdylib_name)#(".dll"));
                    if (name == null) {
                        throw UnsupportedError(#_("This platform is not supported."));
                    }
                    if (name == "") {
                        return Api.loadStatic();
                    } else {
                        return Api.loadDynamic(name);
                    }
                }

                ffi.Pointer<T> allocate<T extends ffi.NativeType>(int byteCount, int alignment) {
                    return _allocate(byteCount, alignment).cast();
                }

                void deallocate<T extends ffi.NativeType>(ffi.Pointer pointer, int byteCount, int alignment) {
                    this._deallocate(pointer.cast(), byteCount, alignment);
                }

                #(for func in iface.functions() => #(self.generate_function(&func)))

                late final _allocatePtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Pointer<ffi.Uint8> Function(ffi.IntPtr, ffi.IntPtr)>>("allocate");

                late final _allocate = _allocatePtr.asFunction<
                    ffi.Pointer<ffi.Uint8> Function(int, int)>();

                late final _deallocatePtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Void Function(ffi.Pointer<ffi.Uint8>, ffi.IntPtr, ffi.IntPtr)>>("deallocate");

                late final _deallocate = _deallocatePtr.asFunction<
                    Function(ffi.Pointer<ffi.Uint8>, int, int)>();

                #(for func in iface.imports(&self.abi) => #(self.generate_wrapper(func)))
            }

            #(for obj in iface.objects() => #(self.generate_object(obj)))

            #(for func in iface.imports(&self.abi) => #(self.generate_return_struct(&func.symbol, &func.ret)))
        }
    }

    fn generate_object(&self, obj: AbiObject) -> dart::Tokens {
        quote! {
            class #(&obj.name) {
                final Api _api;
                final Box _box;

                #(&obj.name)._(this._api, this._box);

                #(for func in &obj.methods => #(self.generate_function(func)))

                void drop() {
                    this._box.drop();
                }
            }
        }
    }

    fn generate_function(&self, func: &AbiFunction) -> dart::Tokens {
        let ffi = self.abi.import(func);
        let api = match &func.ty {
            FunctionType::Constructor(_) => quote!(api),
            FunctionType::Method(_) => quote!(this._api),
            FunctionType::Function => quote!(this),
        };
        let args = quote!(#(for (name, ty) in &func.args => #(self.generate_type(ty)) #name,));
        let body = quote!(#(for instr in &ffi.instr => #(self.generate_instr(&api, instr))));
        match &func.ty {
            FunctionType::Constructor(object) => quote! {
                factory #object.#(&func.name)(Api api, #args) {
                    #body
                }
            },
            _ => {
                let ret = if let Some(ret) = func.ret.as_ref() {
                    self.generate_type(ret)
                } else {
                    quote!(void)
                };
                quote! {
                    #ret #(&func.name)(#args) {
                        #body
                    }
                }
            }
        }
    }

    fn generate_instr(&self, api: &dart::Tokens, instr: &Instr) -> dart::Tokens {
        match instr {
            Instr::BorrowSelf(out) => quote!(final #(self.ident(out)) = this._box.borrow();),
            Instr::BorrowObject(in_, out) => {
                quote!(final #(self.ident(out)) = #(self.ident(in_))._box.borrow();)
            }
            Instr::MoveObject(in_, out)
            | Instr::MoveFuture(in_, out)
            | Instr::MoveStream(in_, out) => {
                quote!(final #(self.ident(out)) = #(self.ident(in_))._box.move();)
            }
            Instr::MakeObject(obj, box_, drop, out) => quote! {
                final ffi.Pointer<ffi.Void> #(self.ident(box_))_0 = ffi.Pointer.fromAddress(#(self.ident(box_)));
                final #(self.ident(box_))_1 = new Box(#api, #(self.ident(box_))_0, #_(#drop));
                #(self.ident(box_))_1._finalizer = _registerFinalizer(#(self.ident(box_))_1);
                final #(self.ident(out)) = new #obj._(#api, #(self.ident(box_))_1);
            },
            Instr::BindArg(arg, out) => quote!(final #(self.ident(out)) = #arg;),
            Instr::BindRet(ret, idx, out) => {
                quote!(final #(self.ident(out)) = #(self.ident(ret)).#(format!("arg{}", idx));)
            }
            Instr::LowerNum(in_, out, _num) | Instr::LiftNum(in_, out, _num) => {
                quote!(final int #(self.ident(out)) = #(self.ident(in_));)
            }
            Instr::LowerBool(in_, out) => {
                quote!(final #(self.ident(out)) = #(self.ident(in_)) ? 1 : 0;)
            }
            Instr::LiftBool(in_, out) => {
                quote!(final #(self.ident(out)) = #(self.ident(in_)) > 0;)
            }
            Instr::StrLen(in_, out) | Instr::VecLen(in_, out) => {
                quote!(final #(self.ident(out)) = #(self.ident(in_)).length;)
            }
            Instr::Allocate(ptr, len, size, align) => {
                quote!(final #(self.ident(ptr)) = #api.allocate(#(self.ident(len)) * #(*size), #(*align)).address;)
            }
            Instr::Deallocate(ptr, len, size, align) => quote! {
                if (#(self.ident(len)) > 0) {
                    final ffi.Pointer<ffi.Void> #(self.ident(ptr))_0;
                    #(self.ident(ptr))_0 = ffi.Pointer.fromAddress(#(self.ident(ptr)));
                    #api.deallocate(#(self.ident(ptr))_0, #(self.ident(len)) * #(*size), #(*align));
                }
            },
            Instr::LowerString(in_, ptr, len) => quote! {
                final ffi.Pointer<ffi.Uint8> #(self.ident(ptr))_0 = ffi.Pointer.fromAddress(#(self.ident(ptr)));
                final #(self.ident(in_))_0 = utf8.encode(#(self.ident(in_)));
                final Uint8List #(self.ident(in_))_1 = #(self.ident(ptr))_0.asTypedList(#(self.ident(len)));
                #(self.ident(in_))_1.setAll(0, #(self.ident(in_))_0);
            },
            Instr::LiftString(ptr, len, out) => quote! {
                final ffi.Pointer<ffi.Uint8> #(self.ident(ptr))_0 = ffi.Pointer.fromAddress(#(self.ident(ptr)));
                final #(self.ident(out)) = utf8.decode(#(self.ident(ptr))_0.asTypedList(#(self.ident(len))));
            },
            Instr::LowerVec(in_, ptr, len, ty) => quote! {
                final ffi.Pointer<#(self.generate_native_num_type(*ty))> #(self.ident(ptr))_0 =
                    ffi.Pointer.fromAddress(#(self.ident(ptr)));
                final #(self.ident(in_))_1 = #(self.ident(ptr))_0.asTypedList(#(self.ident(len)));
                #(self.ident(in_))_1.setAll(0, #(self.ident(in_)));
            },
            Instr::LiftVec(ptr, len, out, ty) => quote! {
                final ffi.Pointer<#(self.generate_native_num_type(*ty))> #(self.ident(ptr))_0 =
                    ffi.Pointer.fromAddress(#(self.ident(ptr)));
                final #(self.ident(out)) = #(self.ident(ptr))_0.asTypedList(#(self.ident(len))).toList();
            },
            Instr::Call(symbol, ret, args) => quote! {
                final #(self.ident(ret)) = #api.#symbol(#(for arg in args => #(self.ident(arg)),));
            },
            Instr::ReturnValue(ret) => quote!(return #(self.ident(ret));),
            Instr::ReturnVoid => quote!(return;),
        }
    }

    fn ident(&self, ident: &u32) -> dart::Tokens {
        quote!(#(format!("tmp{}", ident)))
    }

    fn generate_wrapper(&self, func: Import) -> dart::Tokens {
        let native_args =
            quote!(#(for (_, ty) in &func.args => #(self.generate_native_num_type(*ty)),));
        let wrapped_args =
            quote!(#(for (_, ty) in &func.args => #(self.generate_wrapped_num_type(*ty)),));
        let native_ret = self.generate_native_return_type(&func.symbol, &func.ret);
        let wrapped_ret = self.generate_wrapped_return_type(&func.symbol, &func.ret);
        let symbol_ptr = format!("{}Ptr", &func.symbol);
        quote! {
            late final #(&symbol_ptr)  =
                _lookup<ffi.NativeFunction<#native_ret Function(#native_args)>>(#_(#(&func.symbol)));

            late final #(&func.symbol) =
                #symbol_ptr.asFunction<#wrapped_ret Function(#wrapped_args)>();
        }
    }

    fn generate_type(&self, ty: &AbiType) -> dart::Tokens {
        match ty {
            AbiType::Num(ty) => self.generate_wrapped_num_type(*ty),
            AbiType::Isize | AbiType::Usize => quote!(int),
            AbiType::Bool => quote!(bool),
            AbiType::RefStr | AbiType::String => quote!(String),
            AbiType::RefSlice(ty) | AbiType::Vec(ty) => {
                quote!(List<#(self.generate_wrapped_num_type(*ty))>)
            }
            AbiType::Object(ty) | AbiType::RefObject(ty) => quote!(#ty),
            AbiType::Option(_) => todo!(),
            AbiType::Result(_) => todo!(),
            AbiType::Future(_) => todo!(),
            AbiType::Stream(_) => todo!(),
        }
    }

    fn generate_wrapped_num_type(&self, ty: NumType) -> dart::Tokens {
        match ty {
            NumType::F32 | NumType::F64 => quote!(double),
            _ => quote!(int),
        }
    }

    fn generate_native_num_type(&self, ty: NumType) -> dart::Tokens {
        match ty {
            NumType::I8 => quote!(ffi.Int8),
            NumType::I16 => quote!(ffi.Int16),
            NumType::I32 => quote!(ffi.Int32),
            NumType::I64 => quote!(ffi.Int64),
            NumType::U8 => quote!(ffi.Uint8),
            NumType::U16 => quote!(ffi.Uint16),
            NumType::U32 => quote!(ffi.Uint32),
            NumType::U64 => quote!(ffi.Uint64),
            NumType::F32 => quote!(ffi.Float),
            NumType::F64 => quote!(ffi.Double),
        }
    }

    fn generate_native_return_type(&self, symbol: &str, ret: &[NumType]) -> dart::Tokens {
        match ret.len() {
            0 => quote!(ffi.Void),
            1 => self.generate_native_num_type(ret[0]),
            _ => quote!(#(format!("{}Return", symbol))),
        }
    }

    fn generate_wrapped_return_type(&self, symbol: &str, ret: &[NumType]) -> dart::Tokens {
        match ret.len() {
            0 => quote!(void),
            1 => self.generate_wrapped_num_type(ret[0]),
            _ => quote!(#(format!("{}Return", symbol))),
        }
    }

    fn generate_return_struct(&self, symbol: &str, ret: &[NumType]) -> dart::Tokens {
        if ret.len() < 2 {
            quote!()
        } else {
            quote! {
                class #(format!("{}Return", symbol)) extends ffi.Struct {
                    #(for (i, ty) in ret.iter().enumerate() => #(self.generate_return_struct_field(i, *ty)))
                }
            }
        }
    }

    fn generate_return_struct_field(&self, i: usize, ty: NumType) -> dart::Tokens {
        quote! {
            @#(self.generate_native_num_type(ty))()
            external #(self.generate_wrapped_num_type(ty)) #(format!("arg{}", i));
        }
    }
}

#[cfg(feature = "test_runner")]
pub mod test_runner {
    use super::*;
    use crate::{Abi, RustGenerator};
    use anyhow::Result;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use trybuild::TestCases;

    pub fn compile_pass(iface: &str, rust: rust::Tokens, dart: dart::Tokens) -> Result<()> {
        let iface = Interface::parse(iface)?;
        let mut rust_file = NamedTempFile::new()?;
        let mut rust_gen = RustGenerator::new(Abi::native());
        let rust_tokens = rust_gen.generate(iface.clone());
        let mut dart_file = NamedTempFile::new()?;
        let dart_gen = DartGenerator::new("compile_pass".to_string());
        let dart_tokens = dart_gen.generate(iface);

        let library_tokens = quote! {
            #rust_tokens
            #rust
        };

        let bin_tokens = quote! {
            #dart_tokens

            extension on List {
                bool equals(List list) {
                    if (this.length != list.length) return false;
                    for (int i = 0; i < this.length; i++) {
                        if (this[i] != list[i]) {
                            return false;
                        }
                    }
                    return true;
                }
            }

            void main() {
                final api = Api.load();
                #dart
            }
        };

        let library = library_tokens.to_file_string()?;
        rust_file.write_all(library.as_bytes())?;
        let bin = bin_tokens.to_file_string()?;
        dart_file.write_all(bin.as_bytes())?;

        let library_dir = tempfile::tempdir()?;
        let library_file = library_dir.as_ref().join("libcompile_pass.so");
        let runner_tokens: rust::Tokens = quote! {
            fn main() {
                use std::process::Command;
                let ret = Command::new("rustc")
                    .arg("--crate-name")
                    .arg("compile_pass")
                    .arg("--crate-type")
                    .arg("cdylib")
                    .arg("-o")
                    .arg(#(quoted(library_file.as_path().to_str().unwrap())))
                    .arg(#(quoted(rust_file.as_ref().to_str().unwrap())))
                    .status()
                    .unwrap()
                    .success();
                assert!(ret);
                //println!("{}", #_(#bin));
                let ret = Command::new("dart")
                    .env("LD_LIBRARY_PATH", #(quoted(library_dir.as_ref().to_str().unwrap())))
                    .arg("--enable-asserts")
                    //.arg("--observe")
                    //.arg("--write-service-info=service.json")
                    .arg(#(quoted(dart_file.as_ref().to_str().unwrap())))
                    .status()
                    .unwrap()
                    .success();
                assert!(ret);
            }
        };

        let mut runner_file = NamedTempFile::new()?;
        let runner = runner_tokens.to_file_string()?;
        runner_file.write_all(runner.as_bytes())?;

        let test = TestCases::new();
        test.pass(runner_file.as_ref());
        Ok(())
    }
}
