use heck::SnakeCase;
use quote::Tokens;
use syn::{Attribute, Fields, FieldsNamed, Ident, Item, ItemStruct, Visibility};

pub fn quote(state: Item) -> Tokens {
    match state {
        Item::Struct(ref struct_item) => {
            quote_struct_item(struct_item)
        }
        Item::Fn(_) => {
            quote! {}
        }
        _ => {
            panic!("The attribute is only allowed for fns and structs");
        }
    }
}

fn quote_struct_item(item: &ItemStruct) -> Tokens {
    let state_ident = &item.ident;
    let comp_ident = &Ident::from(format!("{}Component", item.ident));
    let state_fields = &item.fields;
    let vis = &item.vis;
    let augmented_state = quote_augmented_state(item.attrs.clone(), vis, state_ident, state_fields);
    let assert_lifecycle = assert_lifecycle(state_ident);
    let comp_struct = quote_new_struct(vis, comp_ident, state_ident);
    let component_of = impl_component_of(comp_ident, state_ident);
    let component_impl = quote_component_impl(comp_ident, state_ident, state_fields);
    let lifecycle_impl = impl_lifecycle_for_comp(comp_ident);
    let state_setters = impl_state_setters(state_ident, state_fields);
    quote! {
        #augmented_state

        #assert_lifecycle

        #comp_struct

        #component_of

        #component_impl

        #lifecycle_impl

        #state_setters
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
        }
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        }
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
        }
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        }
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

fn impl_state_setters(state: &Ident, fields: &Fields) -> Tokens {
    match *fields {
        Fields::Named(ref named_fields) => {
            let named = &named_fields.named;
            let mut setters = vec![];
            for field in named.iter() {
                let ident = field.ident.as_ref().unwrap();
                let fn_name = Ident::from(format!("set_{}", ident));
                let ty = &field.ty;
                setters.push(
                    quote! {
                        fn #fn_name(&mut self, value: #ty) {
                            self.#ident = value;
                            self.notifier();
                        }
                    }
                );
            }
            quote! {
                impl #state {
                    #(#setters)*
                }
            }
        },
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        },
        Fields::Unit => {
            quote!()
        }
    }
}