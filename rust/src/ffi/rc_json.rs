use serde_json::*;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(PartialEq, Debug)]
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

macro_rules! rc_json {
    ($($json:tt)+) => {
        Rc::new(json!($($json)+).into())
    };
}
