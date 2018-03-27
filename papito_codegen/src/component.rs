use heck::SnakeCase;
use quote::Tokens;
use syn::{Attribute, Fields, FieldsNamed, Ident, Item, ItemStruct, Visibility, Type};

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
    let comp_struct = quote_new_struct(vis, comp_ident, state_ident, state_fields);
    let component_of = impl_component_of(comp_ident, state_ident);
    let component_impl = quote_component_impl(comp_ident, state_ident, state_fields);
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
    }
}

fn impl_component_of(comp: &Ident, state: &Ident) -> Tokens {
    quote! {
        impl ::papito_dom::ComponentOf for #state {
            type Comp = #comp;
        }
    }
}

fn quote_new_struct(vis: &Visibility, comp_ident: &Ident, state_ident: &Ident, fields: &Fields) -> Tokens {
    let props_type = get_props_type_from_fields(fields);
    if let Some(props_type) = props_type {
        quote! {
            #vis struct #comp_ident {
                inner: ::std::rc::Rc<::std::cell::RefCell<#state_ident>>,
                props: ::std::rc::Rc<#props_type>
            }
        }
    } else {
        quote! {
            #vis struct #comp_ident {
                inner: ::std::rc::Rc<::std::cell::RefCell<#state_ident>>
            }
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
            let named = modify_props_type(fields_named);
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

fn modify_props_type(fields_named: &FieldsNamed) -> Vec<Tokens> {
    let prop_ident = &Ident::from("props".to_string());
    fields_named.named.iter()
        .map(|it| {
            let ident = it.ident.as_ref().unwrap();
            let ty = &it.ty;
            if ident == prop_ident {
                quote! {
                    props: ::std::rc::Rc<#ty>
                }
            } else {
                quote! {
                    #ident: #ty
                }
            }
        }).collect::<Vec<Tokens>>()
}

fn quote_component_impl(comp_ident: &Ident, state_ident: &Ident, fields: &Fields) -> Tokens {
    let create_fn = impl_create_fn(comp_ident, state_ident, fields);
    let update_fn = impl_update_fn(fields);
    let props_fn = impl_props_fn(fields);
    let props_type = get_props_type_from_fields(fields);
    let props_type = if let Some(prop_type) = props_type {
        quote! {
            #prop_type
        }
    } else {
        quote! {
            ()
        }
    };
    quote! {
        impl ::papito_dom::Component for #comp_ident {
            type Props = #props_type;

            #create_fn

            #update_fn

            #props_fn
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
        if ident != &Ident::from("props".to_string()) {
            field_inits.push(quote! {
                #ident: Default::default()
            });
        }
    }
    let has_props = has_props_field(fields);
    if has_props {
        quote! {
            fn create(props: Self::Props, notifier: Box<Fn()>) -> Self {
                let props = ::std::rc::Rc::new(props);
                let state = #state_ident {
                    #(#field_inits),*,
                    props,
                    notifier
                };
                #comp_ident {
                    inner: ::std::rc::Rc::new(::std::cell::RefCell::new(state)),
                    props
                }
            }
        }
    } else {
        quote! {
            fn create(_: Self::Props, notifier: Box<Fn()>) -> Self {
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
    if get_props_type_from_fields(fields).is_some() {
        quote! {
            fn update(&mut self, props: Self::Props) {
                let props = ::std::rc::Rc::new(props);
                self.inner.borrow_mut().props = props.clone();
                self.props = props;
                self.inner.borrow().notify();
            }
        }
    } else {
        quote! {
            fn update(&mut self, _: Self::Props) {}
        }
    }
}

fn impl_props_fn(fields: &Fields) -> Tokens {
    if get_props_type_from_fields(fields).is_some() {
        quote! {
            fn props(&self) -> &Self::Props {
                &*self.props
            }
        }
    } else {
        quote! {
            fn props(&self) -> &Self::Props {
                &()
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

fn get_props_type_from_fields(fields: &Fields) -> Option<Type> {
    match *fields {
        Fields::Named(ref named_fields) => {
            get_props_type(named_fields)
        }
        _ => None
    }
}

fn get_props_type(fields: &FieldsNamed) -> Option<Type> {
    let props_ident = Ident::from("props".to_string());
    for field in fields.named.iter() {
        if field.ident.as_ref().unwrap() == &props_ident {
            return Some(field.ty.clone());
        }
    }
    None
}

fn has_props_field(fields: &FieldsNamed) -> bool {
    get_props_type(fields).is_some()
}