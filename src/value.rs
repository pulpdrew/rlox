use crate::object::{ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjString};
use std::cmp::Ordering;
use std::fmt;
use std::ops;
use std::rc::Rc;

/// A value is anything that can be put on the VM's stack
/// or listed in the executable's constant table
#[derive(Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    Function(Rc<ObjFunction>),
    Closure(Rc<ObjClosure>),
    String(Rc<ObjString>),
    Class(Rc<ObjClass>),
    Instance(Rc<ObjInstance>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Nil => write!(f, "Nil",),
            Value::Function(func) => write!(f, "{:?}", func),
            Value::Closure(c) => write!(f, "{:?}", c),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Class(c) => write!(f, "{:?}", c),
            Value::Instance(i) => write!(f, "{:?}", i),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Function(func) => write!(f, "{}", func),
            Value::Closure(c) => write!(f, "{}", c),
            Value::String(s) => write!(f, "{}", s),
            Value::Class(c) => write!(f, "{}", c),
            Value::Instance(i) => write!(f, "{}", i),
        }
    }
}

impl Value {
    /// Indicates whether the Value is a `Number` variant
    pub fn is_number(&self) -> bool {
        if let Value::Number(..) = self {
            true
        } else {
            false
        }
    }

    /// Indicates whether the Value is a `Bool` variant
    pub fn is_bool(&self) -> bool {
        if let Value::Bool(..) = self {
            true
        } else {
            false
        }
    }

    /// Indicates whether the Value is a `Nil` variant
    pub fn is_nil(&self) -> bool {
        if let Value::Nil = self {
            true
        } else {
            false
        }
    }

    pub fn is_string(&self) -> bool {
        if let Value::String(_) = self {
            true
        } else {
            false
        }
    }

    /// Indicates whether the Value is 'Truthy' according to the rules of the language
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Number(n) => (n - 0f64).abs() > std::f64::EPSILON,
            Value::String(s) => !s.string.is_empty(),
            _ => true,
        }
    }
}

/// Overloads the `+` operator for Values. Only `Number` and `String` variants can be added.
impl ops::Add for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Value {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left + right)
            } else {
                panic!("Attempted to add [Number] + [Not a number]");
            }
        } else if let Value::String(left) = self {
            if let Value::String(right) = rhs {
                Value::from(format!("{}{}", left, right))
            } else {
                panic!("Attempted to add [String] + [Not a String]");
            }
        } else {
            panic!("Attempted apply '+' to something that wasn't a number or string.");
        }
    }
}

/// Overloads the `-` operator for Values. Only `Number` variants can be subtracted.
impl ops::Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Value) -> Value {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left - right)
            } else {
                panic!("Attempted to subtract {:?} - {:?}", self, rhs);
            }
        } else {
            panic!("Attempted to subtract {:?} - {:?}", self, rhs);
        }
    }
}

/// Overloads the `*` operator for Values. Only `Number` variants can be multiplied.
impl ops::Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Value) -> Value {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left * right)
            } else {
                panic!("Attempted to multiply {:?}* {:?}", self, rhs);
            }
        } else {
            panic!("Attempted to multiply {:?} * {:?}", self, rhs);
        }
    }
}

/// Overloads the `/` operator for Values. Only `Number` variants can be divided.
impl ops::Div for Value {
    type Output = Value;

    fn div(self, rhs: Value) -> Value {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left / right)
            } else {
                panic!("Attempted to divide {:?} / {:?}", self, rhs);
            }
        } else {
            panic!("Attempted to divide {:?} / {:?}", self, rhs);
        }
    }
}

/// Overloads the unary `-` operator for Values. Only `Number` variants can be negated.
impl ops::Neg for Value {
    type Output = Value;

    fn neg(self) -> Value {
        if let Value::Number(left) = self {
            Value::Number(-left)
        } else {
            panic!("Attempted to negate {:?}", self);
        }
    }
}

/// Overloads the `==` operator for Values.
impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match self {
            Value::Bool(b) => other.is_truthy() == *b,
            Value::Number(n) => match other {
                Value::Number(o) => n == o,
                _ => false,
            },
            Value::Nil => other.is_nil(),
            Value::Function(l) => match other {
                Value::Function(r) => l == r,
                _ => false,
            },
            Value::Closure(l) => match other {
                Value::Closure(r) => l == r,
                _ => false,
            },
            Value::String(l) => match other {
                Value::String(r) => l.string == r.string,
                _ => false,
            },
            Value::Class(l) => match other {
                Value::Class(r) => l == r,
                _ => false,
            },
            Value::Instance(l) => match other {
                Value::Instance(r) => l == r,
                _ => false,
            },
        }
    }
}

/// Compares Values, if they are `Number` types
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if let Value::Number(n1) = self {
            if let Value::Number(n2) = other {
                if n1 < n2 {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl From<f64> for Value {
    fn from(number: f64) -> Self {
        Value::Number(number)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::String(Rc::new(ObjString::from(string)))
    }
}

impl From<&str> for Value {
    fn from(string: &str) -> Self {
        Value::String(Rc::new(ObjString::from(string)))
    }
}

impl From<ObjFunction> for Value {
    fn from(func: ObjFunction) -> Self {
        Value::Function(Rc::new(func))
    }
}

impl From<ObjClosure> for Value {
    fn from(closure: ObjClosure) -> Self {
        Value::Closure(Rc::new(closure))
    }
}

impl From<ObjClass> for Value {
    fn from(class: ObjClass) -> Self {
        Value::Class(Rc::new(class))
    }
}

impl From<ObjInstance> for Value {
    fn from(instance: ObjInstance) -> Self {
        Value::Instance(Rc::new(instance))
    }
}
