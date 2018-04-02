#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate heck;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Item, DeriveInput};

mod events;
mod common;
mod component;

#[proc_macro_attribute]
pub fn component(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item: Item = syn::parse(input).unwrap();
    component::quote(item).into()
}

#[proc_macro_derive(Lifecycle)]
pub fn derive_lifecycle(input: TokenStream) -> TokenStream {
    let derive: DeriveInput = syn::parse(input).unwrap();
    let ident = &derive.ident;
    let expanded = quote! {
        impl ::papito::prelude::Lifecycle for #ident {}
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn events(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let state: Item = syn::parse(input).unwrap();
    let wrapper = events::quote(&state);
    let expanded = quote! {
        #state

        #wrapper
    };
    expanded.into()
}

// Just a placeholder attribute to be used by `#[events]` otherwise it does not compile
#[proc_macro_attribute]
pub fn event(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    input
}