use syn::{Ident, Type, Path, Item, ItemStruct, Visibility, Attribute, Fields, Field};
use quote::Tokens;

pub fn quote(item: Item) -> Tokens {
    match item {
        Item::Struct(item_struct) => {
            let component_data = ComponentData::parse(&item_struct);
            let mut component_struct = ComponentStruct::parse(&item_struct);
            if let Some(ref comp_data) = component_data {
                component_struct.set_data(comp_data.ident.clone());
            }
            let component_struct = component_struct.quote();

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
    ident: Ident,
    data: Option<Ident>,
}

impl ComponentStruct {
    fn parse(item: &ItemStruct) -> ComponentStruct {
        let attrs = item.attrs.clone();
        let vis = item.vis.clone();
        let ident = item.ident.clone();
        ComponentStruct {
            attrs,
            vis,
            ident,
            data: None,
        }
    }

    fn set_data(&mut self, data: Ident) {
        self.data = Some(data);
    }

    fn quote(self) -> Tokens {
        let attrs = self.attrs;
        let vis = self.vis;
        let ident = self.ident;
        if let Some(data) = self.data {
            quote! {
                #vis struct #ident {
                    _data: ::std::rc::Rc<::std::cell::RefCell<#data>>
                }
            }
        } else {
            quote! {
                #vis struct #ident;
            }
        }
    }
}

struct ComponentData {
    ident: Ident,
    fields: DataFields,
    component: Ident,
}

impl ComponentData {
    fn parse(item: &ItemStruct) -> Option<ComponentData> {
        let data_fields = DataFields::parse(&item.fields);
        if let Some(fields) = data_fields {
            let component = item.ident.clone();
            let ident = Ident::from(format!("{}Data", &component));
            Some(ComponentData {
                ident,
                component,
                fields,
            })
        } else {
            None
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
}

struct DataField {
    ident: Ident,
    ty: Type,
    is_prop: bool
}

impl DataField {
    fn parse(field: &Field) -> DataField {
        if !field.vis.is_private() {
            panic!("Only private fields allowed.");
        }
        DataField {
            ident: field.ident.as_ref().unwrap().clone(),
            ty: field.ty.clone(),
            is_prop: field.attrs.has_prop_attribute()
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