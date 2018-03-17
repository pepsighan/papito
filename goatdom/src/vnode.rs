use velement::VElement;
use vlist::VList;
use vtext::VText;

pub enum VNode {
    Text(VText),
    Element(VElement),
    List(VList)
}