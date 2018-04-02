use syn::{Ident, Type, Item, ItemStruct, Visibility, Attribute};
use quote::Tokens;

pub fn quote(item: Item) -> Tokens {
    match item {
        Item::Struct(item_struct) => {
            let component_data = ComponentData::parse(&item_struct);
            let component_struct = ComponentStruct::parse(&item_struct);

            let component_struct = component_struct.quote();

            quote! {
                #component_struct
            }
        },
        _ => {
            panic!("`#[component]` can only be used on a struct");
        }
    }
}

struct ComponentStruct {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    data: Option<Ident>
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

struct Prop {
    ident: Ident,
    ty: Type
}

struct State {
    ident: Ident,
    ty: Type
}

enum StateOrProp {
    State(State),
    Prop(Prop)
}

struct ComponentData {
    ident: Ident,
    fields: Vec<StateOrProp>
}

impl ComponentData {
    fn parse(item: &ItemStruct) -> ComponentData {
        unimplemented!()
    }
}