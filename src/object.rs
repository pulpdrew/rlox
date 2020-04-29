use std::fmt;

#[derive(Debug, Clone)]
pub enum Obj {
    String(String),
}
#[derive(Debug, Clone)]
pub enum ObjKind {
    String,
}

impl Obj {
    pub fn as_string(&self) -> Result<&String, ()> {
        match self {
            Obj::String(s) => Ok(&s),
        }
    }
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::String(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(feature = "trace_drops")]
impl Drop for Obj {
    fn drop(&mut self) {
        println!("**Dropped {:?}**", self)
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
