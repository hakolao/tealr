//!# Tealr_derive
//!The derive macro used by [tealr](https://github.com/lenscas/tealr/tree/master/tealr).
//!
//!Tealr is a crate that can generate `.d.tl` files for types that are exposed to `lua`/`teal` through [rlua](https://crates.io/crates/rlua)
//!
//!Read the [README.md](https://github.com/lenscas/tealr/tree/master/tealr/README.md) in [tealr](https://github.com/lenscas/tealr/tree/master/tealr) for more information.

#[macro_use]
extern crate quote;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
mod embed_compiler;
#[cfg(feature = "derive")]
mod from_to_lua;
#[cfg(feature = "derive")]
mod user_data;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
use embed_compiler::EmbedOptions;
use proc_macro::TokenStream;
use venial::parse_declaration;

///Implements [rlua::UserData](rlua::UserData) and `tealr::TypeBody`
///
///It wraps the [rlua::UserDataMethods](rlua::UserDataMethods) into `tealr::rlu::UserDataWrapper`
///and then passes it to `tealr::rlu::TealData::add_methods`.
///
///Type body is implemented in a similar way, where it uses the `tealr::TealData` implementation to get the types
#[cfg(feature = "derive")]
#[proc_macro_derive(RluaUserData)]
pub fn rlua_user_data_derive(input: TokenStream) -> TokenStream {
    use user_data::impl_rlua_user_data_derive;

    let ast = parse_declaration(input.into());
    impl_rlua_user_data_derive(&ast).into()
}

///Implements [mlua::UserData](mlua::UserData) and `tealr::TypeBody`
///
///It wraps the [mlua::UserDataMethods](mlua::UserDataMethods) into `tealr::mlu::UserDataWrapper`
///and then passes it to `tealr::TealData::add_methods`.
///
///Type body is implemented in a similar way, where it uses the `tealr::mlu::TealData` implementation to get the types
#[cfg(feature = "derive")]
#[proc_macro_derive(MluaUserData)]
pub fn mlua_user_data_derive(input: TokenStream) -> TokenStream {
    use user_data::impl_mlua_user_data_derive;

    let ast = parse_declaration(input.into());
    impl_mlua_user_data_derive(&ast).into()
}
///Implements `tealr::TypeName`.
///
///`TypeName::get_type_name` will return the name of the rust type.
#[cfg(feature = "derive")]
#[proc_macro_derive(TypeName)]
pub fn type_representation_derive(input: TokenStream) -> TokenStream {
    use user_data::impl_type_representation_derive;

    let ast = parse_declaration(input.into());

    impl_type_representation_derive(&ast).into()
}

///Implement both [rlua::UserData](rlua::UserData) and `tealr::TypeName`.
///
///Look at [tealr_derive::RluaUserData](tealr_derive::RluaUserData) and [tealr_derive::TypeName](tealr_derive::TypeName)
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
#[proc_macro_derive(RluaTealDerive)]
pub fn rlua_teal_derive(input: TokenStream) -> TokenStream {
    use crate::user_data::{impl_rlua_user_data_derive, impl_type_representation_derive};

    let ast = parse_declaration(input.into());
    let mut stream = impl_type_representation_derive(&ast);
    stream.extend(impl_rlua_user_data_derive(&ast));
    stream.into()
}

///Implement both [mlua::UserData](mlua::UserData) and `tealr::TypeName`.
///
///Look at [tealr_derive::MluaUserData](tealr_derive::MluaUserData) and [tealr_derive::TypeName](tealr_derive::TypeName)
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
#[proc_macro_derive(MluaTealDerive)]
pub fn mlua_teal_derive(input: TokenStream) -> TokenStream {
    use crate::user_data::impl_type_representation_derive;
    use user_data::impl_mlua_user_data_derive;

    let ast = parse_declaration(input.into());

    let mut stream = impl_type_representation_derive(&ast);
    stream.extend(impl_mlua_user_data_derive(&ast));
    stream.into()
}

#[cfg(feature = "compile")]
mod compile_inline_teal;
///Compiles the given teal code at compile time to lua.
///
///The macro tries it best to pass the correct `--include-dir` to tl using `CARGO_MANIFEST_DIR`.
///However, this isn't always where you want it to be. In that case you can add an extra argument that will be joined with `CARGO_MANIFEST_DIR` using [std::path::PathBuf::join](std::path::PathBuf#method.join)
///
///## Compile time requirement!
///At this point in time this requires you to have the teal compiler installed and accessible as `tl`.
///
///## Example
///```
///# use tealr_derive::compile_inline_teal;
///assert_eq!(compile_inline_teal!("local a : number = 1"),"local a = 1\n")
///```

#[cfg(feature = "compile")]
#[proc_macro]
pub fn compile_inline_teal(input: TokenStream) -> TokenStream {
    use crate::compile_inline_teal::compile_inline_teal;
    compile_inline_teal(input.into()).into()
}
///Embeds the teal compiler, making it easy to load teal files directly.
///
///It can either download the given version from Github (default), luarocks or uses the compiler already installed on your system
///Compiling it without the lua5.3 compatibility library and embedding it into your application.
///
///It returns a closure that takes the file that needs to run
///and returns valid lua code that both prepares the lua vm so it can run teal files and
///loads the given file using `require`, returning the result of the file that got loaded.
///## NOTE!
///Due to how the teal files are being loaded, they won't be typed checked.
///More info on: [https://github.com/teal-language/tl/blob/master/docs/tutorial.md](https://github.com/teal-language/tl/blob/master/docs/tutorial.md) (Search for "loader")
///
///## Compile time requirement!
///This needs to be able to run `lua` at compile time to compile the teal compiler.
///
///If a local teal compiler is used, then `tl` needs to run at compile time instead.
///
///## Example
///Downloads:
///```rust
///# use tealr_derive::embed_compiler;
/// //This downloads from github
/// let compiler = embed_compiler!("v0.9.0");
///
/// let compiler = embed_compiler!(Github(version = "v0.9.0"));
/// let compiler = embed_compiler!(Luarocks(version = "v0.9.0"));
/// let lua_code = compiler("your_teal_file.tl");
///```
///From filesystem
//Not tested so it can have a nice path and also to not depend on having the teal compiler at a static place.
///```ignore
/// let compiler = embed_compiler!(Local(path = "some/path/to/tl.tl"));
/// //This tries to find the teal compiler on its own
/// let compiler = embed_compiler!(Local());
///```
#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
#[proc_macro]
pub fn embed_compiler(input: TokenStream) -> TokenStream {
    use syn::parse_macro_input;
    let input = parse_macro_input!(input as EmbedOptions);
    let compiler = embed_compiler::get_teal(input);
    let primed_vm_string = format!(
        "local tl = (function()\n{}\nend)()\ntl.loader()\n",
        compiler
    );
    let stream = quote! {
        |require:&str| {
            format!("{}\n return require('{}')",#primed_vm_string,require)
        }
    };
    stream.into()
}
#[cfg(feature = "derive")]
#[proc_macro_derive(MluaFromToLua, attributes(tealr))]
pub fn mlua_from_to_lua(input: TokenStream) -> TokenStream {
    crate::from_to_lua::mlua_from_to_lua(input.into()).into()
}

///Derives [FromLua](rlua::FromLua), [ToLua](rlua::ToLua) and [TypeBody](tealr::TypeBody) for the type.
///Right now it only works for Structs
#[cfg(feature = "derive")]
#[proc_macro_derive(RluaFromToLua, attributes(tealr))]
pub fn rlua_from_to_lua(input: TokenStream) -> TokenStream {
    crate::from_to_lua::rlua_from_to_lua(input.into()).into()
}
