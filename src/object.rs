use crate::executable::Executable;
use std::fmt;
use std::rc::Rc;

pub struct ObjFunction {
    pub arity: u8,
    pub bin: Executable,
    pub name: Box<ObjString>,
}

impl fmt::Display for ObjFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn: {}>", self.name)
    }
}

impl fmt::Debug for ObjFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn: {}>", self.name)
    }
}

impl PartialEq for ObjFunction {
    fn eq(&self, other: &ObjFunction) -> bool {
        self.arity == other.arity && self.name == other.name && self.bin == other.bin
    }
}

#[cfg(feature = "trace_drops")]
impl Drop for ObjFunction {
    fn drop(&mut self) {
        println!("**Dropped [{:?}]**", self)
    }
}

#[derive(PartialEq)]
pub struct ObjClosure {
    pub function: Rc<ObjFunction>,
}

impl fmt::Display for ObjClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn: {}>", self.function.name)
    }
}

impl fmt::Debug for ObjClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn: {}>", self.function.name)
    }
}

#[cfg(feature = "trace_drops")]
impl Drop for ObjClosure {
    fn drop(&mut self) {
        println!("**Dropped [{:?}]**", self)
    }
}

#[derive(PartialEq)]
pub struct ObjString {
    pub string: String,
}

impl fmt::Display for ObjString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl fmt::Debug for ObjString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.string)
    }
}

#[cfg(feature = "trace_drops")]
impl Drop for ObjString {
    fn drop(&mut self) {
        println!("**Dropped [{:?}]**", self)
    }
}

impl From<String> for ObjString {
    fn from(string: String) -> Self {
        ObjString { string }
    }
}

impl From<&str> for ObjString {
    fn from(string: &str) -> Self {
        ObjString {
            string: string.to_string(),
        }
    }
}
