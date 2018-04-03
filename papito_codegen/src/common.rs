use syn::Visibility;

pub trait IsPrivate {
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