use bacon_rajan_cc::{Cc, Trace, Tracer};
use indexmap::IndexMap;
use serde_json::*;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

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
        &mut $gc.borrow_inner_ref_mut() as &mut InnerValue
    };
}

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

impl From<Value> for GCValue {
    fn from(v: Value) -> Self {
        GCValue::new(v.into())
    }
}

impl Trace for GCValue {
    fn trace(&mut self, tracer: &mut Tracer) {
        self.borrow_inner_ref_mut().trace(tracer)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum InnerValue {
    Null,
    Bool(bool),
    Number(value::Number),
    String(String),
    Array(Vec<GCValue>),
    Object(IndexMap<String, Rc<RefCell<GCValue>>>),
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
                    .map(|(k, v)| (k, Rc::new(RefCell::new(GCValue::new(v.into())))))
                    .collect(),
            ),
        }
    }
}

impl Into<Value> for InnerValue {
    fn into(self) -> Value {
        match self {
            InnerValue::Null => Value::Null,
            InnerValue::Bool(b) => Value::Bool(b),
            InnerValue::Number(n) => Value::Number(n),
            InnerValue::String(s) => Value::String(s),
            InnerValue::Array(a) => Value::Array(
                a.into_iter()
                    .map(|e| gc_borrow_inner!(e).clone().into())
                    .collect(),
            ),
            InnerValue::Object(m) => Value::Object(
                m.into_iter()
                    .map(|(k, v)| (k, gc_borrow_inner!(&v.borrow()).clone().into()))
                    .collect(),
            ),
        }
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
