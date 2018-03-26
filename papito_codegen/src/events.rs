use quote::Tokens;
use syn::{Item, Type, Ident, Path, ImplItem, ImplItemMethod, FnArg, ArgSelfRef, ArgCaptured};
use common::{split_path, component_of_state};

pub fn quote(item: &Item) -> Tokens {
    match *item {
        Item::Impl(ref item_impl) => {
            let self_ty = *item_impl.self_ty.clone();
            let path = component_path_of(self_ty);
            impl_wrapper_for_any_events(item_impl.items.clone(), path)
        }
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
        }
        _ => panic!("Only type paths are allowed for impls attributed with `#[events]`")
    }
}

fn impl_wrapper_for_any_events(items: Vec<ImplItem>, comp_path: Path) -> Tokens {
    let mut iter = items.into_iter();
    while let Some(ImplItem::Method(method_item)) = iter.next() {
        if has_event_attribute(&method_item) {
            let (fn_name, self_arg, event_arg) = get_metadata(method_item);
        }
    }
    quote!()
}

fn has_event_attribute(item: &ImplItemMethod) -> bool {
    item.attrs.iter().any(|it| it.path == Path::from(Ident::from("event".to_string())))
}

fn get_metadata(item: ImplItemMethod) -> (Ident, ArgSelfRef, ArgCaptured) {
    let sig = item.sig;
    let fn_name = sig.ident;
    let mut args = sig.decl.inputs.into_iter();
    let first_arg = args.next().expect("This method has no argument");
    let second_arg = args.next().expect("This method should have second argument");
    if let Some(_) = args.next() {
        panic!("This method cannot have any more that 2 arguments");
    }
    let first_arg = if let FnArg::SelfRef(self_arg) = first_arg {
        self_arg
    } else {
        panic!("The first arg can only be `&self` or `&mut self`");
    };
    let second_arg = if let FnArg::Captured(arg) = second_arg {
        arg
    } else {
        panic!("The second arg must be an explicit type");
    };
    (fn_name, first_arg, second_arg)
}