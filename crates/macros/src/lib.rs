extern crate proc_macro;

mod audio_library;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(AudioLibrary, attributes(looped, intro_looped))]
pub fn derive_audio_library(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);

  match &ast.data {
    Data::Enum(_e) => audio_library::derive_audio_library(ast)
      .unwrap_or_else(syn::Error::into_compile_error)
      .into(),
    Data::Struct(_s) => syn::Error::new(ast.ident.span(), "can only be derived for enums")
      .into_compile_error()
      .into(),
    Data::Union(_u) => syn::Error::new(ast.ident.span(), "can only be derived for enums")
      .into_compile_error()
      .into(),
  }
}
