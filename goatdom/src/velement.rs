use CowStr;
use indexmap::IndexMap;
use vnode::VNode;
use std::fmt::Display;
use std::fmt::{Formatter, self};

#[derive(Debug, Eq, PartialEq)]
pub struct ClassString(CowStr);

impl Display for ClassString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, " class=\"{}\"", self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Attributes(IndexMap<CowStr, CowStr>);

impl Display for Attributes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (k, v) in self.0.iter() {
            write!(f, " {}=\"{}\"", k, v)?;
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct VElement {
    tag: CowStr,
    class: Option<ClassString>,
    attrs: Option<Attributes>,
    child: Option<Box<VNode>>,
    is_self_closing: bool,
}

impl VElement {
    pub fn new(tag: CowStr, class: Option<ClassString>, attrs: Option<Attributes>, child: Option<VNode>, is_self_closing: bool) -> VElement {
        VElement {
            tag,
            class,
            attrs,
            child: child.map(|it| Box::new(it)),
            is_self_closing
        }
    }
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
        if self.is_self_closing {
            write!(f, ">")
        } else {
            write!(f, ">")?;
            if let Some(ref child) = self.child {
                write!(f, "{}", child)?;
            }
            write!(f, "</{}>", self.tag)
        }
    }
}

impl<A, B> From<Vec<(A, B)>> for Attributes where
    A: Into<CowStr>,
    B: Into<CowStr> {
    fn from(item: Vec<(A, B)>) -> Self {
        Attributes(item.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect())
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, VNode, bool)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, VNode, bool)) -> Self {
        let tag = item.0.into();
        let mut attrs: Attributes = item.1.into();
        let class = swap_remove_class(&mut attrs);
        VElement::new(tag, class, Some(attrs), Some(item.2), item.3)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, VNode)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, VNode)) -> Self {
        let tag = item.0.into();
        let mut attrs: Attributes = item.1.into();
        let class = swap_remove_class(&mut attrs);
        VElement::new(tag, class, Some(attrs), Some(item.2), false)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>, bool)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>, bool)) -> Self {
        let tag = item.0.into();
        let mut attrs: Attributes = item.1.into();
        let class = swap_remove_class(&mut attrs);
        VElement::new(tag, class, Some(attrs), None, item.2)
    }
}

impl<A, B, C> From<(A, Vec<(B, C)>)> for VElement where
    A: Into<CowStr>,
    B: Into<CowStr>,
    C: Into<CowStr> {
    fn from(item: (A, Vec<(B, C)>)) -> Self {
        let tag = item.0.into();
        let mut attrs: Attributes = item.1.into();
        let class = swap_remove_class(&mut attrs);
        VElement::new(tag, class, Some(attrs), None, false)
    }
}

impl<A> From<(A, bool)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, bool)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, None, item.1)
    }
}

impl<A> From<(A, ())> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, ())) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, None, false)
    }
}

impl<A> From<(A, VNode, bool)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, VNode, bool)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, Some(item.1), item.2)
    }
}

impl<A> From<(A, VNode)> for VElement where
    A: Into<CowStr> {
    fn from(item: (A, VNode)) -> Self {
        let tag = item.0.into();
        VElement::new(tag, None, None, Some(item.1), false)
    }
}

fn swap_remove_class(attrs: &mut Attributes) -> Option<ClassString> {
    attrs.0.swap_remove("class").map(|it| ClassString(it))
}
