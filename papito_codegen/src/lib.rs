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
use heck::SnakeCase;

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
        #component
    };
    expanded.into()
}

fn quote_struct_item(item: &ItemStruct) -> Tokens {
    let state_ident = &item.ident;
    let comp_ident = Ident::from(format!("{}Component", item.ident));
    let assert_mod_ident = Ident::from(format!("{}Assertions", item.ident).to_snake_case());
    let state_fields = &item.fields;
    let vis = &item.vis;
    let augmented_state = quote! {
        #item
    };
    let assert_lifecycle = quote! {
        mod #assert_mod_ident {
            struct _AssertLifecycle where #state_ident: ::papito_dom::Lifecycle;
        }
    };
    let new_struct = quote! {
        #vis struct #comp_ident {
            inner: ::std::rc::Rc<::std::cell::RefCell<#state_ident>>
        }
    };
    let component_of = quote! {
        impl ::papito_dom::ComponentOf for #state_ident {
            type Comp = #comp_ident;
        }
    };
    let component_impl = quote! {
        impl ::papito_dom::Component for #comp_ident {
            type Props = ();

            fn create(props: Self::Props, notifier: Box<Fn()>) -> Self {
                let state = #state_ident;
                #comp_ident {
                    inner: ::std::rc::Rc::new(::std::cell::RefCell::new(state))
                }
            }

            fn update(&mut self, props: Self::Props) {
                unimplemented!();
            }

            fn props(&self) -> &Self::Props {
                unimplemented!();
            }
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
        #augmented_state

        #assert_lifecycle

        #new_struct

        #component_of

        #component_impl

        #lifecycle_impl
    }
}