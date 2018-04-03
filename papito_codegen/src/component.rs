use quote::Tokens;
use syn::{Attribute, Field, Fields, Ident, Item, ItemStruct, Path, Type, Visibility};

pub fn quote(item: Item) -> Tokens {
    match item {
        Item::Struct(item_struct) => {
            let mut component_data = ComponentData::parse(&item_struct);
            component_data.quote()
        }
        _ => {
            panic!("`#[component]` can only be used on a struct");
        }
    }
}

struct ComponentData {
    attrs: Vec<Attribute>,
    vis: Visibility,
    component: Ident,
    data: Option<Ident>,
    props: Option<Ident>,
    fields: DataFields,
}

impl ComponentData {
    fn parse(item: &ItemStruct) -> ComponentData {
        let fields = DataFields::parse(&item.fields);
        let component = item.ident.clone();
        let attrs = item.attrs.clone();
        let vis = item.vis.clone();
        ComponentData {
            attrs,
            vis,
            data: None,
            props: None,
            component,
            fields,
        }
    }

    fn quote(&mut self) -> Tokens {
        let data_struct = self.quote_data_struct();
        let props_struct = self.quote_props_struct();
        let data_impl = self.quote_data_impl();
        let component_struct = self.quote_component_struct();
        let component_impl = self.quote_component_impl();
        let impl_component_trait = self.quote_impl_component_trait();
        quote! {
            #component_struct

            #props_struct

            #data_struct

            #data_impl

            #component_impl

            #impl_component_trait
        }
    }

    fn quote_component_struct(&self) -> Tokens {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let component = &self.component;
        if let Some(data) = self.data {
            quote! {
                #(#attrs)*
                #vis struct #component {
                    _data: ::std::rc::Rc<::std::cell::RefCell<#data>>,
                    _notifier: Box<Fn()>
                }

                impl #component {
                    fn _notify(&self) {
                        (self._notifier)();
                    }
                }
            }
        } else {
            quote! {
                #(#attrs)*
                #vis struct #component;
            }
        }
    }

    fn quote_props_struct(&mut self) -> Tokens {
        let props_fields = self.fields.quote_props_fields();
        if let Some(props_fields) = props_fields {
            let props = Ident::from(format!("_{}Props", &self.component));
            self.props = Some(props);
            let props = self.props.as_ref().unwrap();
            quote! {
                struct #props {
                    #props_fields
                }
            }
        } else {
            quote!()
        }
    }

    fn quote_data_struct(&mut self) -> Tokens {
        let data_fields = self.fields.quote_data_fields();
        if let Some(data_fields) = data_fields {
            let data = Ident::from(format!("_{}Data", &self.component));
            self.data = Some(data);
            let data = self.data.as_ref().unwrap();
            quote! {
                struct #data {
                    #data_fields
                }
            }
        } else {
            quote!()
        }
    }

    fn quote_data_impl(&self) -> Tokens {
        if let Some(ref data) = self.data {
            let data_getters = self.fields.quote_getters();
            let data_setters = self.fields.quote_setters();
            quote! {
                impl #data {
                    #data_getters

                    #data_setters
                }
            }
        } else {
            quote!()
        }
    }

    fn quote_component_impl(&self) -> Tokens {
        if self.data.is_some() {
            let component = &self.component;
            let component_getters = self.fields.quote_component_getters();
            let component_setters = self.fields.quote_component_setters();
            quote! {
                impl #component {
                    #component_getters

                    #component_setters
                }
            }
        } else {
            quote!()
        }
    }

    fn quote_impl_component_trait(&self) -> Tokens {
        let component = &self.component;

        let props_ty = if let Some(ref props) = self.props {
            quote!( #props )
        } else {
            quote!( () )
        };

        let create_fn = self.quote_create_fn();
        let update_fn = self.quote_update_fn();
        let eq_props_fn = self.quote_eq_props_fn();

        quote! {
            impl ::papito_dom::Component for #component {
                type Props = #props_ty;

                #create_fn

                #update_fn

                #eq_props_fn
            }
        }
    }

    fn quote_create_fn(&self) -> Tokens {
        let component = &self.component;
        if let Some(ref data) = self.data {
            let data_init = self.fields.quote_data_init();
            if self.props.is_some() {
                quote! {
                    fn create(props: Self::Props, notifier: Box<Fn()>) -> Self {
                        let _data = #data {
                            #data_init
                        };
                        #component {
                            _data: ::std::rc::Rc::new(::std::cell::RefCell::new(_data)),
                            _notifier: notifier
                        }
                    }
                }
            } else {
                quote! {
                    fn create(_: Self::Props, notifier: Box<Fn()>) -> Self {
                        let _data = #data {
                            #data_init
                        };
                        #component {
                            _data: ::std::rc::Rc::new(::std::cell::RefCell::new(_data)),
                            _notifier: notifier
                        }
                    }
                }
            }
        } else {
            quote! {
                fn create(_: Self::Props, _: Box<Fn()>) -> Self {
                    #component
                }
            }
        }
    }

    fn quote_update_fn(&self) -> Tokens {
        if self.data.is_some() && self.props.is_some() {
            let props_update = self.fields.quote_props_update();
            quote! {
                    fn update(&self, props: Self::Props) {
                        let _data = &mut self._data.borrow_mut();
                        #props_update
                        self._notify();
                    }
                }
        } else {
            quote! {
                fn update(&self, _: Self::Props) {}
            }
        }
    }

    fn quote_eq_props_fn(&self) -> Tokens {
        if self.data.is_some() && self.props.is_some() {
            let props_eq = self.fields.quote_props_eq();
            quote! {
                fn eq_props(&self, props: &Self::Props) -> bool {
                    let _data = &*self._data.borrow();
                    #props_eq
                }
            }
        } else {
            // If there are no props, then the components have the same props
            quote! {
                fn eq_props(&self, _: &Self::Props) -> bool {
                    true
                }
            }
        }
    }
}

struct DataFields {
    fields: Vec<DataField>
}

impl DataFields {
    fn parse(fields: &Fields) -> DataFields {
        match *fields {
            Fields::Unit => DataFields { fields: vec![] },
            Fields::Unnamed(_) => {
                panic!("Tuple structs are not allowed as a `#[component]`")
            }
            Fields::Named(ref named_fields) => {
                let fields = named_fields.named.iter()
                    .map(|field| DataField::parse(field))
                    .collect();
                DataFields {
                    fields
                }
            }
        }
    }

    fn quote_data_fields(&self) -> Option<Tokens> {
        let fields: Vec<_> = self.fields.iter()
            .map(|it| it.quote_data_field())
            .collect();
        if !fields.is_empty() {
            Some(quote! {
                #(#fields),*
            })
        } else {
            None
        }
    }

    fn quote_props_fields(&self) -> Option<Tokens> {
        let fields: Vec<_> = self.fields.iter()
            .map(|it| it.quote_props_field())
            .filter(|it| it.is_some())
            .map(|it| it.unwrap())
            .collect();
        if !fields.is_empty() {
            Some(quote! {
                #(#fields),*
            })
        } else {
            None
        }
    }

    fn quote_getters(&self) -> Tokens {
        let getters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_getters())
            .collect();
        quote! {
            #(#getters)*
        }
    }

    fn quote_component_getters(&self) -> Tokens {
        let getters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_component_getters())
            .collect();
        quote! {
            #(#getters)*
        }
    }

    fn quote_setters(&self) -> Tokens {
        let setters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_setters())
            .collect();
        quote! {
            #(#setters)*
        }
    }

    fn quote_component_setters(&self) -> Tokens {
        let setters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_component_setters())
            .collect();
        quote! {
            #(#setters)*
        }
    }

    fn quote_data_init(&self) -> Tokens {
        let inits: Vec<_> = self.fields.iter()
            .map(|it| it.quote_data_init())
            .collect();
        quote! {
            #(#inits),*
        }
    }

    fn quote_props_update(&self) -> Tokens {
        let updates: Vec<_> = self.fields.iter()
            .map(|it| it.quote_props_update())
            .collect();
        quote! {
            #(#updates)*
        }
    }

    fn quote_props_eq(&self) -> Tokens {
        let eqs: Vec<_> = self.fields.iter()
            .map(|it| it.quote_props_eq())
            .filter(|it| it.is_some())
            .map(|it| it.unwrap())
            .collect();
        quote! {
            #(#eqs) && *
        }
    }
}

struct DataField {
    ident: Ident,
    ty: Type,
    is_prop: bool,
}

impl DataField {
    fn parse(field: &Field) -> DataField {
        if !field.vis.is_private() {
            panic!("Only private fields allowed.");
        }
        DataField {
            ident: field.ident.as_ref().unwrap().clone(),
            ty: field.ty.clone(),
            is_prop: field.attrs.has_prop_attribute(),
        }
    }

    fn quote_data_field(&self) -> Tokens {
        let ident = &self.ident;
        let ty = &self.ty;
        quote! {
            #ident: #ty
        }
    }

    fn quote_props_field(&self) -> Option<Tokens> {
        if self.is_prop {
            let ident = &self.ident;
            let ty = &self.ty;
            Some(quote! {
                #ident: #ty
            })
        } else {
            None
        }
    }

    fn quote_getters(&self) -> Tokens {
        let ident = &self.ident;
        let ty = &self.ty;
        quote! {
            fn #ident(&self) -> #ty {
                self.#ident.clone()
            }
        }
    }

    fn quote_component_getters(&self) -> Tokens {
        let ident = &self.ident;
        let ty = &self.ty;
        quote! {
            fn #ident(&self) -> #ty {
                self._data.borrow().#ident()
            }
        }
    }

    fn quote_setters(&self) -> Option<Tokens> {
        if self.is_prop {
            let ident = &self.ident;
            let ty = &self.ty;
            Some(quote! {
                fn set_#ident(&mut self, value: #ty) -> bool {
                    if self.#ident != value {
                        self.#ident = value;
                        true
                    } else {
                        false
                    }
                }
            })
        } else {
            None
        }
    }

    fn quote_component_setters(&self) -> Option<Tokens> {
        if self.is_prop {
            let ident = &self.ident;
            let ty = &self.ty;
            Some(quote! {
                fn set_#ident(&self, value: #ty) {
                    let changed = self._data.borrow_mut().set_#ident(value);
                    if changed {
                        self._notify();
                    }
                }
            })
        } else {
            None
        }
    }

    fn quote_data_init(&self) -> Tokens {
        let ident = &self.ident;
        if self.is_prop {
            quote! {
                #ident: props.#ident
            }
        } else {
            quote! {
                #ident: Default::default()
            }
        }
    }

    fn quote_props_update(&self) -> Tokens {
        if self.is_prop {
            let ident = &self.ident;
            quote! {
                _data.#ident = props.#ident;
            }
        } else {
            quote!()
        }
    }

    fn quote_props_eq(&self) -> Option<Tokens> {
        if self.is_prop {
            let ident = &self.ident;
            Some(quote! {
                _data.#ident == props.#ident
            })
        } else {
            None
        }
    }
}

trait HasPropAttribute {
    fn has_prop_attribute(&self) -> bool;
}

impl HasPropAttribute for Vec<Attribute> {
    fn has_prop_attribute(&self) -> bool {
        self.iter().any(|it| it.has_prop_attribute())
    }
}

impl HasPropAttribute for Attribute {
    fn has_prop_attribute(&self) -> bool {
        if self.path == Path::from(Ident::from(format!("prop"))) {
            if !self.tts.is_empty() {
                panic!("No arguments supported");
            }
            true
        } else {
            false
        }
    }
}

trait IsPrivate {
    fn is_private(&self) -> bool;
}

impl IsPrivate for Visibility {
    fn is_private(&self) -> bool {
        match *self {
            Visibility::Inherited => true,
            _ => false
        }
    }
}