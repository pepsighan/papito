use quote::Tokens;
use syn::{Item, Type, Ident, Path, ImplItem, ImplItemMethod};
use common::{split_path, component_of_state};

pub fn quote(item: &Item) -> Tokens {
    match *item {
        Item::Impl(ref item_impl) => {
            let self_ty = *item_impl.self_ty.clone();
            let path = component_path_of(self_ty);
            impl_wrapper_for_any_events(item_impl.items.clone(), path)
        },
        _ => {
            panic!("`#[events]` attribute can only be used with an impl block");
        }
    }
}

fn component_path_of(self_ty: Type) -> Path {
    match self_ty {
        Type::Path(type_path) => {
            let (mut path, mut last_segment) = split_path(type_path);
            last_segment.ident = component_of_state(&last_segment.ident);
            path.segments.push(last_segment);
            path
        },
        _ => panic!("Only type paths are allowed for impls attributed with `#[events]`")
    }
}

fn impl_wrapper_for_any_events(items: Vec<ImplItem>, comp_path: Path) -> Tokens {
    let mut iter = items.into_iter();
    while let Some(ImplItem::Method(method_item)) = iter.next() {
        if has_event_attribute(&method_item) {

        }
    }
}

fn has_event_attribute(item: &ImplItemMethod) -> bool {
    item.attrs.iter().any(|it| it.path == Path::from(Ident::from("event".to_string())))
}