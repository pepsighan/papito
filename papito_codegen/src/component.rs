use heck::SnakeCase;
use quote::Tokens;
use syn::{Attribute, Field, Fields, FieldsNamed, Ident, Item, ItemStruct, Visibility, Path};

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
    let props_ident = &Ident::from(format!("{}Prop", item.ident));
    let comp_ident = &Ident::from(format!("{}Component", item.ident));
    let state_fields = &item.fields;
    let vis = &item.vis;
    let props_struct = generate_props_struct(props_ident, state_fields);
    let augmented_state = augment_state_struct(item.attrs.clone(), vis, state_ident, state_fields);
    let assert_lifecycle = assert_lifecycle(state_ident);
    let comp_struct = generate_component_struct(vis, comp_ident, state_ident);
    let component_of = impl_component_of(comp_ident, state_ident);
    let component_impl = impl_component_trait(comp_ident, state_ident, props_ident, state_fields);
    let lifecycle_impl = impl_lifecycle_for_comp(comp_ident);
    let state_setters = impl_state_setters_and_notifier(state_ident, state_fields);
    quote! {
        #augmented_state

        #assert_lifecycle

        #comp_struct

        #component_of

        #component_impl

        #lifecycle_impl

        #state_setters

        #props_struct
    }
}

fn impl_component_of(comp: &Ident, state: &Ident) -> Tokens {
    quote! {
        impl ::papito_dom::ComponentOf for #state {
            type Comp = #comp;
        }
    }
}

fn generate_component_struct(vis: &Visibility, comp_ident: &Ident, state_ident: &Ident) -> Tokens {
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

fn augment_state_struct(attrs: Vec<Attribute>, vis: &Visibility, state_ident: &Ident, fields: &Fields) -> Tokens {
    let notifier = Ident::from("notifier".to_string());
    match *fields {
        Fields::Named(ref fields_named) => {
            let named = sanitize_fields(fields_named);
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
                #vis struct #state_ident;
            }
        }
    }
}

fn sanitize_fields(fields_named: &FieldsNamed) -> Vec<Field> {
    let prop_path = Path::from(Ident::from("prop".to_string()));
    fields_named.named.clone().into_iter()
        .map(|mut it| {
            it.attrs = it.attrs.into_iter().filter(|attr| attr.path != prop_path)
                .collect();
            it
        }).collect::<Vec<Field>>()
}

fn impl_component_trait(comp_ident: &Ident, state_ident: &Ident, props_ident: &Ident, fields: &Fields) -> Tokens {
    let create_fn = impl_create_fn(comp_ident, state_ident, fields);
    let update_fn = impl_update_fn(fields);
    let props_eq_fn = impl_eq_props_fn(fields);
    quote! {
        impl ::papito_dom::Component for #comp_ident {
            type Props = #props_ident;

            #create_fn

            #update_fn

            #props_eq_fn
        }
    }
}

fn impl_create_fn(comp_ident: &Ident, state_ident: &Ident, fields: &Fields) -> Tokens {
    match *fields {
        Fields::Named(ref fields_named) => {
            quote_fields_named(comp_ident, state_ident, fields_named)
        }
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        }
        Fields::Unit => {
            quote_unit_field(comp_ident, state_ident)
        }
    }
}

fn quote_fields_named(comp_ident: &Ident, state_ident: &Ident, fields: &FieldsNamed) -> Tokens {
    let mut field_inits = vec![];
    for field in fields.named.iter() {
        let ident = &field.ident.unwrap();
        if !is_prop_field(field) {
            field_inits.push(quote! {
                #ident: Default::default()
            });
        } else {
            field_inits.push(quote! {
                #ident: props.#ident
            });
        }
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
        fn create(_: Self::Props, _: Box<Fn()>) -> Self {
            let state = #state_ident;
            #comp_ident {
                inner: ::std::rc::Rc::new(::std::cell::RefCell::new(state))
            }
        }
    }
}

fn impl_update_fn(fields: &Fields) -> Tokens {
    let prop_fields = get_props_from_fields(fields);
    if !prop_fields.is_empty() {
        let props_tokens = prop_fields.iter().map(|it| {
            let ident = &it.ident;
            quote! {
                inner.#ident = props.#ident;
            }
        }).collect::<Vec<_>>();
        quote! {
            fn update(&mut self, props: Self::Props) {
                let inner = &mut *self.inner.borrow_mut();
                #(#props_tokens)*
                self.inner.borrow().notify();
            }
        }
    } else {
        quote! {
            fn update(&mut self, _: Self::Props) {}
        }
    }
}

fn impl_eq_props_fn(fields: &Fields) -> Tokens {
    let prop_fields = get_props_from_fields(fields);
    if !prop_fields.is_empty() {
        let comparisons = prop_fields.iter()
            .map(|it| {
                let ident = &it.ident;
                quote! {
                    inner.#ident == rhs.#ident
                }
            })
            .collect::<Vec<_>>();
        quote! {
            fn eq_props(&self, rhs: &Self::Props) -> bool {
                let inner = &*self.inner.borrow();
                #(#comparisons) && *
            }
        }
    } else {
        quote! {
            fn props(&self, _: &Self::Props) -> bool {
                false
            }
        }
    }
}

fn impl_state_setters_and_notifier(state: &Ident, fields: &Fields) -> Tokens {
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
                        #[allow(dead_code)]
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

                    #[allow(dead_code)]
                    fn notifier(&self) {
                        let notifier = &self.notifier;
                        notifier();
                    }
                }
            }
        }
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        }
        Fields::Unit => {
            quote!()
        }
    }
}

fn generate_props_struct(ident: &Ident, fields: &Fields) -> Tokens {
    let props = get_props_from_fields(fields);
    let field_tokens = props.iter().map(|it| {
        let ident = it.ident;
        let ty = &it.ty;
        quote! {
            #ident: #ty
        }
    }).collect::<Vec<_>>();
    if field_tokens.is_empty() {
        quote! {
            struct #ident;
        }
    } else {
        quote! {
            struct #ident {
                #(#field_tokens),*
            }
        }
    }
}

fn get_props_from_fields(fields: &Fields) -> Vec<Field> {
    match *fields {
        Fields::Named(ref named_fields) => {
            let mut list = vec![];
            for field in named_fields.named.iter() {
                if is_prop_field(field) {
                    list.push(field.clone());
                }
            }
            list
        }
        Fields::Unnamed(_) => {
            panic!("Tuple structs are not supported as components");
        }
        Fields::Unit => {
            Vec::new()
        }
    }
}

fn is_prop_field(field: &Field) -> bool {
    let prop_path = Path::from(Ident::from("prop".to_string()));
    field.attrs.iter().any(|it| it.path == prop_path)
}