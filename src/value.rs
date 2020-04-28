use crate::object::Obj;
use std::cmp::Ordering;
use std::fmt;
use std::ops;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    Obj(Obj),
}

impl Value {
    pub fn is_number(&self) -> bool {
        if let Value::Number(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_bool(&self) -> bool {
        if let Value::Bool(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_nil(&self) -> bool {
        if let Value::Nil = self {
            true
        } else {
            false
        }
    }

    pub fn is_obj(&self) -> bool {
        if let Value::Obj(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            Value::Number(n) => (n != &0f64),
            Value::Obj(_) => true,
        }
    }
}

impl ops::Add for Value {
    type Output = Value;

    fn add(self, rhs: Value) -> Value {
        if let Value::Number(left) = self {
            if let Value::Number(right) = rhs {
                Value::Number(left + right)
            } else {
                panic!("Attempted to add [Number] + [Not a number]");
            }
        } else if let Value::Obj(Obj::String(left)) = self {
            if let Value::Obj(Obj::String(right)) = rhs {
                Value::Obj(Obj::String(Rc::new(format!("{}{}", left, right))))
            } else {
                panic!("Attempted to add [String] + [Not a String]");
            }
        } else {
            panic!("Attempted apply '+' to something that wasn't a number or string.");
        }
    }
}

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

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match self {
            Value::Bool(b) => other.is_truthy() == *b,
            Value::Number(n) => match other {
                Value::Number(o) => n == o,
                _ => false,
            },
            Value::Nil => other.is_nil(),
            Value::Obj(_) => other.is_obj(),
        }
    }
}

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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Obj(o) => write!(f, "{}", o),
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
