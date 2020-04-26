use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Obj {
    String(Rc<String>),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::String(s) => write!(f, "{}", s),
        }
    }
}
