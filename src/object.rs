use crate::executable::Executable;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct ObjFunction {
    pub arity: u8,
    pub bin: Executable,
    pub name: Box<Obj>,
}

#[derive(Debug)]
pub struct ObjClosure {
    pub function: Rc<ObjFunction>,
}

pub enum Obj {
    String(String),
    Function(Rc<ObjFunction>),
    Closure(ObjClosure),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjKind {
    String,
    Function,
    Closure,
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
            Obj::Closure(c) => write!(f, "<fn: {}>", (*c.function).name),
        }
    }
}

impl fmt::Debug for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::String(s) => write!(f, "\"{}\"", s),
            Obj::Function(func) => write!(f, "<fn: {}>", func.name),
            Obj::Closure(c) => write!(f, "<fn: {}>", c.function.name),
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
        Obj::Function(Rc::new(func))
    }
}

impl From<ObjClosure> for Obj {
    fn from(closure: ObjClosure) -> Self {
        Obj::Closure(closure)
    }
}
