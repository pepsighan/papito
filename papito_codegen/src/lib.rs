#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate heck;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Item, Ident, ItemStruct};
use quote::Tokens;

#[proc_macro_attribute]
pub fn component(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let state: Item = syn::parse(input).expect("Expected it to be an Item");
    let component = match state {
        Item::Struct(ref struct_item) => {
            quote_struct_item(struct_item)
        }
        Item::Fn(_) => {
            quote! {}
        }
        _ => {
            panic!("The attribute is only allowed for fns and structs");
        }
    };
    let expanded = quote! {
        #state

        #component
    };
    expanded.into()
}

fn quote_struct_item(item: &ItemStruct) -> Tokens {
    let state_ident = &item.ident;
    let comp_ident = Ident::from(format!("{}Component", item.ident));
    let state_fields = &item.fields;
    let vis = &item.vis;
    let new_struct = quote! {
        #vis struct #comp_ident {
            inner: ::std::rc::Rc<::std::cell::RefCell<#state_ident>>
        }
    };
    let component_of = quote! {
        impl ::papito_dom::ComponentOf for #comp_ident {
            type Comp = #state_ident;
        }
    };
    let lifecycle_impl = quote! {
        impl ::papito::prelude::Lifecycle for #comp_ident {
            fn created(&mut self) {
                self.inner.borrow_mut().created();
            }

            fn mounted(&mut self) {
                self.inner.borrow_mut().mounted();
            }

            fn updated(&mut self) {
                self.inner.borrow_mut().updated();
            }

            fn destroyed(&mut self) {
                self.inner.borrow_mut().destroyed();
            }
        }
    };
    quote! {
        #new_struct

        #component_of

        #lifecycle_impl
    }
}