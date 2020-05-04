use crate::executable::Executable;
use std::fmt;

#[derive(Debug)]
pub struct ObjFunction {
    pub arity: u8,
    pub bin: Executable,
    pub name: Box<Obj>,
}

pub enum Obj {
    String(String),
    Function(ObjFunction),
}
#[derive(Debug, Clone)]
pub enum ObjKind {
    String,
    Function,
}

impl Obj {
    pub fn as_string(&self) -> Result<&String, ()> {
        match self {
            Obj::String(s) => Ok(&s),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::String(s) => write!(f, "{}", s),
            Obj::Function(func) => write!(f, "<fn: {}>", func.name),
        }
    }
}

impl fmt::Debug for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::String(s) => write!(f, "\"{}\"", s),
            Obj::Function(func) => write!(f, "<fn: {}>", func.name),
        }
    }
}

#[cfg(feature = "trace_drops")]
impl Drop for Obj {
    fn drop(&mut self) {
        println!("**Dropped [{:?}]**", self)
    }
}

impl From<String> for Obj {
    fn from(string: String) -> Self {
        Obj::String(string)
    }
}

impl From<&str> for Obj {
    fn from(string: &str) -> Self {
        Obj::String(String::from(string))
    }
}

impl From<ObjFunction> for Obj {
    fn from(func: ObjFunction) -> Self {
        Obj::Function(func)
    }
}
