use syn::{Path, Ident, TypePath, PathSegment};
use syn::punctuated::Pair;

pub fn split_path(type_path: TypePath) -> (Path, PathSegment) {
    let TypePath { qself, mut path } = type_path;
    assert!(qself.is_none(), "No self-type allowed on the concrete type");
    let last_segment = path.segments.pop().unwrap();
    let last_segment = match last_segment {
        Pair::End(segment) => {
            segment
        }
        _ => unreachable!()
    };
    (path, last_segment)
}

pub fn component_of_state(state: &Ident) -> Ident {
    Ident::from(format!("{}Component", state))
}