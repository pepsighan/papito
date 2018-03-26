#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate heck;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::{Item, Ident, Visibility, Attribute, ItemStruct, ItemImpl, DeriveInput, Type, TypePath,
          Path, Fields, FieldsNamed};
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
    let comp_ident = &Ident::from(format!("{}Component", item.ident));
    let state_fields = &item.fields;
    let vis = &item.vis;
    let augmented_state = quote_augmented_state(item.attrs.clone(), vis, state_ident, state_fields);
    let assert_lifecycle = assert_lifecycle(state_ident);
    let new_struct = quote_new_struct(vis, comp_ident, state_ident);
    let component_of = impl_component_of(comp_ident, state_ident);
    let component_impl = quote_component_impl(comp_ident, state_ident, state_fields);
    let lifecycle_impl = impl_lifecycle_for_comp(comp_ident);
    quote! {
        #augmented_state

        #assert_lifecycle

        #new_struct

        #component_of

        #component_impl

        #lifecycle_impl
    }
}

fn impl_component_of(comp: &Ident, state: &Ident) -> Tokens {
    quote! {
        impl ::papito_dom::ComponentOf for #state {
            type Comp = #comp;
        }
    }
}

fn quote_new_struct(vis: &Visibility, comp_ident: &Ident, state_ident: &Ident) -> Tokens {
    quote! {
        #vis struct #comp_ident {
            inner: ::std::rc::Rc<::std::cell::RefCell<#state_ident>>
        }
    }
}

fn assert_lifecycle(state: &Ident) -> Tokens {
    let mod_ = Ident::from(format!("{}StateAssertions", state).to_snake_case());
    quote! {
        mod #mod_ {
            struct _AssertLifecycle where #state: ::papito_dom::Lifecycle;
        }
    }
}

fn impl_lifecycle_for_comp(comp: &Ident) -> Tokens {
    quote! {
        impl ::papito::prelude::Lifecycle for #comp {
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
    }
}

fn quote_augmented_state(attrs: Vec<Attribute>, vis: &Visibility, state_ident: &Ident, fields: &Fields) -> Tokens {
    let notifier = Ident::from("notifier".to_string());
    match *fields {
        Fields::Named(ref fields_named) => {
            let named = &fields_named.named;
            quote! {
                #(#attrs)*
                #vis struct #state_ident {
                    #(#named),*,
                    #notifier: Box<Fn()>
                }
            }
        },
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        },
        Fields::Unit => {
            quote! {
                #(#attrs)*
                #vis struct #state_ident {
                    #notifier: Box<Fn()>
                }
            }
        }
    }
}

fn quote_component_impl(comp_ident: &Ident, state_ident: &Ident, fields: &Fields) -> Tokens {
    let create_fn = match *fields {
        Fields::Named(ref fields_named) => {
            quote_fields_named(comp_ident, state_ident, fields_named)
        },
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        },
        Fields::Unit => {
            quote_unit_field(comp_ident, state_ident)
        }
    };
    quote! {
        impl ::papito_dom::Component for #comp_ident {
            type Props = ();

            #create_fn

            fn update(&mut self, props: Self::Props) {
                unimplemented!();
            }

            fn props(&self) -> &Self::Props {
                unimplemented!();
            }
        }
    }
}

fn quote_fields_named(comp_ident: &Ident, state_ident: &Ident, fields: &FieldsNamed) -> Tokens {
    let mut field_inits = vec![];
    for field in fields.named.iter() {
        let ident = &field.ident.unwrap();
        field_inits.push(quote! {
            #ident: Default::default()
        });
    }
    quote! {
        fn create(props: Self::Props, notifier: Box<Fn()>) -> Self {
            let state = #state_ident {
                #(#field_inits),*,
                notifier
            };
            #comp_ident {
                inner: ::std::rc::Rc::new(::std::cell::RefCell::new(state))
            }
        }
    }
}

fn quote_unit_field(comp_ident: &Ident, state_ident: &Ident) -> Tokens {
    quote! {
        fn create(props: Self::Props, notifier: Box<Fn()>) -> Self {
            let state = #state_ident {
                notifier
            };
            #comp_ident {
                inner: ::std::rc::Rc::new(::std::cell::RefCell::new(state))
            }
        }
    }
}

#[proc_macro_attribute]
pub fn render(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let item: Item = syn::parse(input).expect("Expected it to be an Item");
    let new_impl = match item {
        Item::Impl(item_impl) => {
            impl_render(item_impl)
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

fn impl_render(item_impl: ItemImpl) -> Tokens {
    let (_, trait_, _) = item_impl.trait_
        .expect("The `#[render]` attribute is only allowed on `papito::prelude::Render` trait impl block");
    let self_ty = *item_impl.self_ty;
    let (comp_ty, assert_mod_ident) = match self_ty.clone() {
        Type::Path(type_path) => {
            modify_state_path_to_component_path(type_path)
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

fn modify_state_path_to_component_path(type_path: TypePath) -> (Path, Ident) {
    let TypePath { qself, mut path } = type_path;
    assert!(qself.is_some(), "No self-type allowed on the concrete type");
    let last_segment = path.segments.pop().unwrap();
    let (last_segment, assert_mod_ident) = match last_segment {
        Pair::End(mut segment) => {
            let (comp_ident, assert_mod_ident) = generate_ident(&segment.ident);
            segment.ident = comp_ident;
            (segment, assert_mod_ident)
        },
        _ => unreachable!()
    };
    path.segments.push(last_segment);
    (path, assert_mod_ident)
}

fn generate_ident(ident: &Ident) -> (Ident, Ident) {
    let assert_mod_ident = Ident::from(format!("{}RenderAssertions", ident).to_snake_case());
    let comp_ident = Ident::from(format!("{}Component", ident));
    (comp_ident, assert_mod_ident)
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