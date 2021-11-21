use crate::{AbiFunction, Interface, Type};
use genco::prelude::*;
use genco::tokens::static_literal;

pub struct DartGenerator {
    cdylib_name: String,
}

impl DartGenerator {
    pub fn new(cdylib_name: String) -> Self {
        Self { cdylib_name }
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

            class _Slice extends ffi.Struct {
                @ffi.IntPtr()
                external int ptr;

                @ffi.IntPtr()
                external int len;
            }

            class _Alloc extends ffi.Struct {
                @ffi.IntPtr()
                external int ptr;

                @ffi.IntPtr()
                external int len;

                @ffi.IntPtr()
                external int cap;
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

                late final _allocatePtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Pointer<ffi.Uint8> Function(ffi.IntPtr, ffi.IntPtr)>>("allocate");

                late final _allocate = _allocatePtr.asFunction<
                    ffi.Pointer<ffi.Uint8> Function(int, int)>();

                ffi.Pointer<T> allocate<T extends ffi.NativeType>(int byteCount, int alignment) {
                    return _allocate(byteCount, alignment).cast();
                }

                late final _deallocatePtr = _lookup<
                    ffi.NativeFunction<
                        ffi.Void Function(ffi.Pointer<ffi.Uint8>, ffi.IntPtr, ffi.IntPtr)>>("deallocate");

                late final _deallocate = _deallocatePtr.asFunction<
                    Function(ffi.Pointer<ffi.Uint8>, int, int)>();

                void deallocate<T extends ffi.NativeType>(ffi.Pointer pointer, int byteCount, int alignment) {
                    this._deallocate(pointer.cast(), byteCount, alignment);
                }

                #(for func in iface.into_functions() => #(self.generate_function(func)))
            }
        }
    }

    fn generate_function(&self, func: AbiFunction) -> dart::Tokens {
        quote! {
            #(self.generate_return(func.ret.as_ref())) #(&func.name)(
                #(for (name, ty) in &func.args => #(self.generate_arg(name, ty))))
            {
                #(for (name, ty) in &func.args => #(self.generate_lower(name, ty)))
                final ret = #(format!("_{}", &func.name))(
                    #(for (name, ty) in &func.args => #(self.generate_lower_args(name, ty))));
                #(for (name, ty) in &func.args => #(self.generate_lower_cleanup(name, ty)))
                #(self.generate_lift(func.ret.as_ref()))
            }

            late final #(format!("_{}Ptr", &func.name)) = _lookup<
                ffi.NativeFunction<
                    #(self.generate_return_native(func.ret.as_ref()))
                        Function(#(for (name, ty) in &func.args => #(self.generate_arg_native(name, ty))))>>(
                            #_(#(format!("__{}", &func.name))));

            late final #(format!("_{}", &func.name)) = #(format!("_{}Ptr", &func.name))
                .asFunction<#(self.generate_return_wrapped(func.ret.as_ref()))
                    Function(#(for (name, ty) in &func.args => #(self.generate_arg_wrapped(name, ty))))>();
        }
    }

    fn generate_arg(&self, name: &str, ty: &Type) -> dart::Tokens {
        match ty {
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Usize
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::Isize => quote!(int #name,),
            Type::Bool => quote!(bool #name,),
            Type::F32 | Type::F64 => quote!(double #name,),
            Type::Ref(inner) => match &**inner {
                Type::String => quote!(String #name,),
                arg => todo!("arg &{:?}", arg),
            },
            arg => todo!("arg {:?}", arg),
        }
    }

    fn generate_arg_native(&self, _name: &str, ty: &Type) -> dart::Tokens {
        match ty {
            Type::U8 => quote!(ffi.Uint8,),
            Type::U16 => quote!(ffi.Uint16,),
            Type::U32 => quote!(ffi.Uint32,),
            Type::U64 => quote!(ffi.Uint64,),
            Type::Usize => quote!(ffi.IntPtr,),
            Type::I8 => quote!(ffi.Int8,),
            Type::I16 => quote!(ffi.Int16,),
            Type::I32 => quote!(ffi.Int32,),
            Type::I64 => quote!(ffi.Int64,),
            Type::Isize => quote!(ffi.IntPtr,),
            Type::Bool => quote!(ffi.Uint8,),
            Type::F32 => quote!(ffi.Float,),
            Type::F64 => quote!(ffi.Double,),
            Type::Ref(inner) => match &**inner {
                Type::String => quote!(ffi.Pointer<ffi.Uint8>, ffi.IntPtr,),
                arg => todo!("arg &{:?}", arg),
            },
            arg => todo!("arg {:?}", arg),
        }
    }

    fn generate_arg_wrapped(&self, _name: &str, ty: &Type) -> dart::Tokens {
        match ty {
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Usize
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::Isize => quote!(int),
            Type::Bool => quote!(int),
            Type::F32 | Type::F64 => quote!(double),
            Type::Ref(inner) => match &**inner {
                Type::String => quote!(ffi.Pointer<ffi.Uint8>, int,),
                arg => todo!("arg &{:?}", arg),
            },
            arg => todo!("arg {:?}", arg),
        }
    }

    fn generate_lower(&self, name: &str, ty: &Type) -> dart::Tokens {
        match ty {
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Usize
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::Isize
            | Type::F32
            | Type::F64 => quote!(),
            Type::Bool => quote!(final int #(name)_int = #name ? 1 : 0;),
            Type::Ref(inner) => match &**inner {
                Type::String => quote! {
                    final #(name)_utf8 = utf8.encode(#(name));
                    final int #(name)_len = #(name)_utf8.length;
                    final ffi.Pointer<ffi.Uint8> #(name)_ptr = this.allocate(#(name)_len, 1);
                    final Uint8List #(name)_view = #(name)_ptr.asTypedList(#(name)_len);
                    #(name)_view.setAll(0, #(name)_utf8);
                },
                arg => todo!("arg &{:?}", arg),
            },
            arg => todo!("lower arg {:?}", arg),
        }
    }

    fn generate_lower_args(&self, name: &str, ty: &Type) -> dart::Tokens {
        match ty {
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Usize
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::Isize
            | Type::F32
            | Type::F64 => quote!(#name),
            Type::Bool => quote!(#(name)_int,),
            Type::Ref(inner) => match &**inner {
                Type::String => quote!(#(name)_ptr, #(name)_len,),
                arg => todo!("arg &{:?}", arg),
            },
            arg => todo!("lower arg {:?}", arg),
        }
    }

    fn generate_lower_cleanup(&self, name: &str, ty: &Type) -> dart::Tokens {
        match ty {
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Usize
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::Isize
            | Type::Bool
            | Type::F32
            | Type::F64 => quote!(),
            Type::Ref(inner) => match &**inner {
                Type::String => quote!(this.deallocate(#(name)_ptr, #(name)_len, 1);),
                arg => todo!("arg &{:?}", arg),
            },
            arg => todo!("lower arg {:?}", arg),
        }
    }

    fn generate_lift(&self, ret: Option<&Type>) -> dart::Tokens {
        if let Some(ret) = ret {
            match ret {
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::Isize
                | Type::F32
                | Type::F64 => quote!(return ret;),
                Type::Bool => quote!(return ret > 0;),
                arg => todo!("lift arg {:?}", arg),
            }
        } else {
            quote!()
        }
    }

    fn generate_return(&self, ret: Option<&Type>) -> dart::Tokens {
        if let Some(ret) = ret {
            match ret {
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::Isize => quote!(int),
                Type::Bool => quote!(bool),
                Type::F32 | Type::F64 => quote!(double),
                ret => todo!("ret {:?}", ret),
            }
        } else {
            quote!(void)
        }
    }

    fn generate_return_native(&self, ret: Option<&Type>) -> dart::Tokens {
        if let Some(ret) = ret {
            match ret {
                Type::U8 => quote!(ffi.Uint8),
                Type::U16 => quote!(ffi.Uint16),
                Type::U32 => quote!(ffi.Uint32),
                Type::U64 => quote!(ffi.Uint64),
                Type::Usize => quote!(ffi.IntPtr),
                Type::I8 => quote!(ffi.Int8),
                Type::I16 => quote!(ffi.Int16),
                Type::I32 => quote!(ffi.Int32),
                Type::I64 => quote!(ffi.Int64),
                Type::Isize => quote!(ffi.IntPtr),
                Type::Bool => quote!(ffi.Uint8),
                Type::F32 => quote!(ffi.Float),
                Type::F64 => quote!(ffi.Double),
                ret => todo!("ret {:?}", ret),
            }
        } else {
            quote!(ffi.Void)
        }
    }

    fn generate_return_wrapped(&self, ret: Option<&Type>) -> dart::Tokens {
        if let Some(ret) = ret {
            match ret {
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::Isize => quote!(int),
                Type::Bool => quote!(int),
                Type::F32 | Type::F64 => quote!(double),
                ret => todo!("ret {:?}", ret),
            }
        } else {
            quote!(void)
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
        let rust_gen = RustGenerator::new(Abi::Native);
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
                let ret = Command::new("dart")
                    .env("LD_LIBRARY_PATH", #(quoted(library_dir.as_ref().to_str().unwrap())))
                    .arg("run")
                    .arg("--enable-asserts")
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
