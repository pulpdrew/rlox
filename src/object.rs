use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
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

impl From<String> for Obj {
    fn from(string: String) -> Self {
        Obj::String(Rc::new(string))
    }
}

impl From<&str> for Obj {
    fn from(string: &str) -> Self {
        Obj::String(Rc::new(String::from(string)))
    }
}
