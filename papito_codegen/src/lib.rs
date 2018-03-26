#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate heck;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Item, DeriveInput};

mod component;
mod render;
mod events;

#[proc_macro_attribute]
pub fn component(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let state: Item = syn::parse(input).unwrap();
    let component = component::quote(state);
    let expanded = quote! {
        #component
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn render(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item: Item = syn::parse(input).unwrap();
    let new_impl = render::quote(item);
    let expanded = quote! {
        #new_impl
    };
    expanded.into()
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
    let event = events::quote(state);
    let expanded = quote! {
        #event
    };
    expanded.into()
}