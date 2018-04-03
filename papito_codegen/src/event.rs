use syn::{Item, Ident, Block, Type, ItemFn, ReturnType, FnArg};
use quote::Tokens;
use syn::spanned::Spanned;
use common::IsPrivate;
use proc_macro2::Span;

pub fn quote(item: Item) -> Tokens {
    match item {
        Item::Fn(item_fn) => {
            let ev_data = EventData::parse(item_fn);
            ev_data.quote()
        }
        _ => {
            panic!("`#[event]` only allowed on fns");
        }
    }
}

struct EventData {
    ident: Ident,
    event_ty: Type,
    block: Box<Block>,
    span: Span,
}

impl EventData {
    fn parse(item: ItemFn) -> EventData {
        if !item.vis.is_private() {
            panic!("Event methods can only be a private method");
        }
        if item.unsafety.is_some() {
            panic!("Event methods cannot be unsafe");
        }
        let ident = item.ident;
        let span = item.span();
        let decl = *item.decl;
        if decl.output != ReturnType::Default {
            panic!("Event methods have no return type");
        }
        let mut inputs = decl.inputs.into_iter();
        let first_arg = inputs.next()
            .expect("Event methods must have two arguments first `&self` and second an Event type");
        let second_arg = inputs.next()
            .expect("Event methods must have second argument of an Event type");
        match first_arg {
            FnArg::SelfRef(self_ref) => {
                if self_ref.mutability.is_some() {
                    panic!("`&mut self` not allowed on event method. Use `&self` instead.");
                }
            }
            _ => {
                panic!("First argument of event method must be of type `&self`");
            }
        };
        let event_ty = match second_arg {
            FnArg::Captured(captured) => {
                captured.ty
            }
            _ => {
                panic!("Second argument of event method must be an explicit type");
            }
        };
        let block = item.block;
        EventData {
            ident,
            event_ty,
            block,
            span,
        }
    }

    fn quote(self) -> Tokens {
        let ident = self.ident;
        let event_ty = self.event_ty;
        let block = self.block;
        let span = self.span;
        quote_spanned! { span =>
            fn #ident<'a>(&'a self) -> Fn(#event_ty) + 'a {
                move |ev: #event_ty| {
                    #block
                }
            }
        }
    }
}