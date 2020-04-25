use crate::object::Obj;
use std::cmp::Ordering;
use std::ops;
use std::rc::Rc;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    Obj(Rc<Obj>),
}

#[allow(dead_code)]
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
        } else {
            panic!("Attempted to add [Not a number] + [??}");
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