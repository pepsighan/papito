#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate heck;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Item, Ident, ItemStruct, Type, TypePath};
use syn::punctuated::Pair;
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

#[proc_macro_attribute]
pub fn render(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item: Item = syn::parse(input).expect("Expected it to be an Item");
    let new_impl = match item {
        Item::Impl(item_impl) => {
            let (_, trait_, _) = item_impl.trait_.expect("The `#[render]` attribute is only allowed on `papito::prelude::Render` trait impl block");
            let self_ty = *item_impl.self_ty;
            let (comp_ty, assert_mod_ident) = match self_ty.clone() {
                Type::Path(TypePath { qself, mut path }) => {
                    if qself.is_some() {
                        panic!("No self-type allowed on the concrete type");
                    }
                    let mut last_segment = path.segments.pop().unwrap();
                    let (last_segment, assert_mod_ident) = match last_segment {
                        Pair::End(mut segment) => {
                            let assert_mod_ident = Ident::from(format!("{}RenderAssertions", &segment.ident).to_snake_case());
                            segment.ident = Ident::from(format!("{}Component", segment.ident));
                            (segment, assert_mod_ident)
                        },
                        _ => unreachable!()
                    };
                    path.segments.push(last_segment);
                    (path, assert_mod_ident)
                }
                _ => {
                    panic!("Only type paths are allowed to be implemented by `::papito::prelude::Render`");
                }
            };
            let impl_items = item_impl.items;
            quote! {
                mod #assert_mod_ident {
                    struct _AssertLifecycle where #self_ty: ::papito::prelude::Lifecycle;
                    struct _AssertComponent where #comp_ty: ::papito_dom::Component;
                }

                impl #trait_ for #comp_ty {
                    #(#impl_items)*
                }

                impl #trait_ for #self_ty {
                    fn render(&self) -> ::papito_dom::prelude::VNode {
                        unimplemented!()
                    }
                }
            }
        }
        _ => {
            panic!("The `#[render]` attribute is only allowed for impl blocks");
        }
    };
    let expanded = quote! {
        #new_impl
    };
    expanded.into()
}