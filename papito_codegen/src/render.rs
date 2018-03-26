use heck::SnakeCase;
use quote::Tokens;
use syn::{Ident, Item, ItemImpl, Path, Type, TypePath};
use syn::punctuated::Pair;

pub fn quote(item: Item) -> Tokens {
    match item {
        Item::Impl(item_impl) => {
            impl_render(item_impl)
        }
        _ => {
            panic!("The `#[render]` attribute is only allowed for impl blocks");
        }
    }
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