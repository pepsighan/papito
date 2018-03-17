use CowStr;
use indexmap::IndexMap;
use vnode::VNode;
use std::fmt::Display;
use std::fmt::{Formatter, self};

pub struct ClassString(CowStr);

impl Display for ClassString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, " class=\"{}\"", self.0)
    }
}

pub struct Attributes(IndexMap<CowStr, CowStr>);

impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (k, v) in self.0.iter() {
            write!(f, " {}=\"{}\"", k, v)?;
        }
        Ok(())
    }
}

pub struct VElement {
    tag: CowStr,
    class: Option<ClassString>,
    attrs: Option<Attributes>,
    child: Option<Box<VNode>>
}

impl Display for VElement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<{}", self.tag)?;
        if let Some(ref class) = self.class {
            write!(f, "{}", class)?;
        }
        if let Some(ref attrs) = self.attrs {
            write!(f, "{}", attrs)?;
        }
        write!(f, ">")?;
        if let Some(ref child) = self.child {
            write!(f, "{}", child)?;
        }
        write!(f, "</{}>", self.tag)
    }
}