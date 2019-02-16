use serde_json::*;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub enum RcValue {
    Null,
    Bool(bool),
    Number(value::Number),
    String(String),
    Array(Vec<Rc<RcValue>>),
    Object(HashMap<String, Rc<RcValue>>),
}

impl From<Value> for RcValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => RcValue::Null,
            Value::Bool(b) => RcValue::Bool(b),
            Value::Number(n) => RcValue::Number(n),
            Value::String(s) => RcValue::String(s),
            Value::Array(v) => RcValue::Array(v.into_iter().map(|e| Rc::new(e.into())).collect()),
            Value::Object(m) => {
                RcValue::Object(m.into_iter().map(|(k, v)| (k, Rc::new(v.into()))).collect())
            }
        }
    }
}

impl From<RcValue> for Value {
    fn from(v: RcValue) -> Self {
        match v {
            RcValue::Null => Value::Null,
            RcValue::Bool(b) => Value::Bool(b),
            RcValue::Number(n) => Value::Number(n),
            RcValue::String(s) => Value::String(s),
            RcValue::Array(v) => Value::Array(v.into_iter().map(|e| (*e).clone().into()).collect()),
            RcValue::Object(m) => Value::Object(
                m.into_iter()
                    .map(|(k, v)| (k, (*v).clone().into()))
                    .collect(),
            ),
        }
    }
}

macro_rules! rc_json {
    ($($json:tt)+) => {
        Rc::new(json!($($json)+).into())
    };
}
