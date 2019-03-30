use bacon_rajan_cc::{Cc, Trace, Tracer};
use core::borrow::Borrow;
use indexmap::IndexMap;
use serde_json::*;
use std::cell::{Ref, RefCell, RefMut};

custom_derive! {
    #[derive(Debug, Clone, PartialEq, NewtypeDerefMut, NewtypeDeref, NewtypeFrom)]
    pub struct GCValue(Cc<RefCell<InnerValue>>);
}

impl GCValue {
    pub fn new(v: InnerValue) -> GCValue {
        GCValue {
            0: Cc::new(RefCell::new(v)),
        }
    }

    pub fn borrow_inner_ref(&self) -> Ref<InnerValue> {
        (&*self.0 as &RefCell<InnerValue>).borrow()
    }

    pub fn borrow_inner_ref_mut(&mut self) -> RefMut<InnerValue> {
        (&*self.0 as &RefCell<InnerValue>).borrow_mut()
    }
}

impl Trace for InnerValue {
    fn trace(&mut self, tracer: &mut Tracer) {
        match self {
            InnerValue::Null => {}
            InnerValue::Bool(_) => {}
            InnerValue::Number(_) => {}
            InnerValue::String(_) => {}
            InnerValue::Array(v) => v.trace(tracer),
            InnerValue::Object(v) => {
                for (_, v) in v {
                    v.trace(tracer)
                }
            }
        }
    }
}

impl Trace for GCValue {
    fn trace(&mut self, tracer: &mut Tracer) {
        self.borrow_inner_ref_mut().trace(tracer)
    }
}

#[derive(PartialEq, Debug)]
pub enum InnerValue {
    Null,
    Bool(bool),
    Number(value::Number),
    String(String),
    Array(Vec<GCValue>),
    Object(IndexMap<String, GCValue>),
}

impl From<Value> for InnerValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => InnerValue::Null,
            Value::Bool(b) => InnerValue::Bool(b),
            Value::Number(n) => InnerValue::Number(n),
            Value::String(s) => InnerValue::String(s),
            Value::Array(v) => {
                InnerValue::Array(v.into_iter().map(|e| GCValue::new(e.into())).collect())
            }
            Value::Object(m) => InnerValue::Object(
                m.into_iter()
                    .map(|(k, v)| (k, GCValue::new(v.into())))
                    .collect(),
            ),
        }
    }
}

impl From<Value> for GCValue {
    fn from(v: Value) -> Self {
        GCValue::new(v.into())
    }
}

#[macro_export]
macro_rules! gc_json {
    ($($json:tt)+) => {
        GCValue::new(json!($($json)+).into())
    };
}

#[macro_export]
macro_rules! gc_borrow_inner {
    ($gc:expr) => {
        &$gc.borrow_inner_ref() as &InnerValue
    };
}

#[macro_export]
macro_rules! gc_borrow_inner_mut {
    ($gc:expr) => {
        $gc.borrow_inner_ref_mut().borrow_mut() as &mut InnerValue
    };
}
