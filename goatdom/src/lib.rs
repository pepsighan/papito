extern crate indexmap;

use std::borrow::Cow;

type CowStr = Cow<'static, str>;

mod vnode;
mod vtext;
mod velement;
mod vlist;