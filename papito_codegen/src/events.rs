use quote::Tokens;
use syn::Item;

pub fn quote(item: Item) -> Tokens {
    match item {
        Item::Impl(item_impl) => {
            quote!{}
        },
        _ => {
            panic!("`#[events]` attribute can only be used with an impl block");
        }
    }
}