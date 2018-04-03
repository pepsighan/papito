use quote::Tokens;
use syn::{Attribute, Field, Fields, Ident, Item, ItemStruct, Path, Type, Visibility};

pub fn quote(item: Item) -> Tokens {
    match item {
        Item::Struct(item_struct) => {
            let component_data = ComponentData::parse(&item_struct);
            let mut component_struct = ComponentStruct::parse(&item_struct);
            if let Some(ref comp_data) = component_data {
                component_struct.set_data(comp_data.data.clone());
            }
            let component_struct = component_struct.quote();
            let component_data = component_data.map(|it| it.quote());

            quote! {
                #component_struct
            }
        }
        _ => {
            panic!("`#[component]` can only be used on a struct");
        }
    }
}

struct ComponentStruct {
    attrs: Vec<Attribute>,
    vis: Visibility,
    component: Ident,
    data: Option<Ident>,
}

impl ComponentStruct {
    fn parse(item: &ItemStruct) -> ComponentStruct {
        let attrs = item.attrs.clone();
        let vis = item.vis.clone();
        let component = item.ident.clone();
        ComponentStruct {
            attrs,
            vis,
            component,
            data: None,
        }
    }

    fn set_data(&mut self, data: Ident) {
        self.data = Some(data);
    }

    fn quote(self) -> Tokens {
        let attrs = self.attrs;
        let vis = self.vis;
        let component = self.component;
        if let Some(data) = self.data {
            quote! {
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
                #vis struct #component;
            }
        }
    }
}

struct ComponentData {
    data: Ident,
    props: Ident,
    fields: DataFields,
    component: Ident,
}

impl ComponentData {
    fn parse(item: &ItemStruct) -> Option<ComponentData> {
        let data_fields = DataFields::parse(&item.fields);
        if let Some(fields) = data_fields {
            let component = item.ident.clone();
            let data = Ident::from(format!("_{}Data", &component));
            let props = Ident::from(format!("_{}Props", &component));
            Some(ComponentData {
                data,
                props,
                component,
                fields,
            })
        } else {
            None
        }
    }

    fn quote(self) -> Tokens {
        let data = self.data;
        let props = self.props;
        let component = self.component;
        let data_fields = self.fields.quote_data();
        let props_fields = self.fields.quote_props();
        let data_getters = self.fields.quote_getters();
        let component_getters = self.fields.quote_component_getters();
        let data_setters = self.fields.quote_setters();
        let component_setters = self.fields.quote_component_setters();

        quote! {
            struct #props {
                #props_fields
            }

            struct #data {
                #data_fields

                #data_setters
            }

            impl #data {
                #data_getters
            }

            impl #component {
                #component_getters

                #component_setters
            }
        }
    }
}

struct DataFields {
    fields: Vec<DataField>
}

impl DataFields {
    fn parse(fields: &Fields) -> Option<DataFields> {
        match *fields {
            Fields::Unit => None,
            Fields::Unnamed(_) => {
                panic!("Tuple structs are not allowed as a `#[component]`")
            }
            Fields::Named(ref named_fields) => {
                let fields = named_fields.named.iter()
                    .map(|field| DataField::parse(field))
                    .collect();
                Some(DataFields {
                    fields
                })
            }
        }
    }

    fn quote_data(&self) -> Tokens {
        let fields: Vec<_> = self.fields.iter()
            .map(|it| it.quote_data())
            .collect();
        quote! {
            #(#fields),*
        }
    }

    fn quote_props(&self) -> Tokens {
        let fields: Vec<_> = self.fields.iter()
            .map(|it| it.quote_props())
            .collect();
        quote! {
            #(#fields),*
        }
    }

    fn quote_getters(&self) -> Tokens {
        let getters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_getters())
            .collect();
        quote! {
            #(#getters),*
        }
    }

    fn quote_component_getters(&self) -> Tokens {
        let getters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_component_getters())
            .collect();
        quote! {
            #(#getters),*
        }
    }

    fn quote_setters(&self) -> Tokens {
        let setters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_setters())
            .collect();
        quote! {
            #(#setters),*
        }
    }

    fn quote_component_setters(&self) -> Tokens {
        let setters: Vec<_> = self.fields.iter()
            .map(|it| it.quote_component_setters())
            .collect();
        quote! {
            #(#setters),*
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

    fn quote_data(&self) -> Tokens {
        let ident = &self.ident;
        let ty = &self.ty;
        quote! {
            #ident: #ty
        }
    }

    fn quote_props(&self) -> Option<Tokens> {
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
            let ty  = &self.ty;
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