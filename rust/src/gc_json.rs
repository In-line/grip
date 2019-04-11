use bacon_rajan_cc::{collect_cycles, number_of_roots_buffered, Cc, Trace, Tracer};
use indexmap::IndexMap;
use serde_json::*;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use std::time::{Duration, Instant};

#[macro_export]
macro_rules! gc_json {
    ($($json:tt)+) => {
        json!($($json)+).into()
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

    pub fn borrow_inner_ref_mut(&self) -> RefMut<InnerValue> {
        (&*self.0 as &RefCell<InnerValue>).borrow_mut()
    }

    pub fn into_with_recursion_limit(self, recursion_limit: usize) -> Value {
        self.borrow_inner_ref()
            .clone()
            .into_with_recursion_limit(recursion_limit)
    }

    pub fn deep_clone_with_recursion_limit(&self, recursion_limit: usize) -> GCValue {
        GCValue::new(match &self.borrow_inner_ref() as &InnerValue {
            InnerValue::Array(a) => InnerValue::Array(
                a.iter()
                    .filter(|_| recursion_limit - 1 != 0)
                    .map(|e| e.deep_clone_with_recursion_limit(recursion_limit - 1))
                    .collect(),
            ),
            InnerValue::Object(m) => InnerValue::Object(
                m.into_iter()
                    .filter(|_| recursion_limit - 1 != 0)
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            Rc::new(RefCell::new(
                                v.borrow()
                                    .deep_clone_with_recursion_limit(recursion_limit - 1),
                            )),
                        )
                    })
                    .collect(),
            ),
            v => v.clone(),
        })
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
    Object(IndexMap<String, Rc<RefCell<GCValue>>, fnv::FnvBuildHasher>),
}

impl From<Value> for GCValue {
    fn from(v: Value) -> Self {
        GCValue::new(match v {
            Value::Null => InnerValue::Null,
            Value::Bool(b) => InnerValue::Bool(b),
            Value::Number(n) => InnerValue::Number(n),
            Value::String(s) => InnerValue::String(s),
            Value::Array(v) => InnerValue::Array(v.into_iter().map(|e| e.into()).collect()),
            Value::Object(m) => InnerValue::Object(
                m.into_iter()
                    .map(|(k, v)| (k, Rc::new(RefCell::new(v.into()))))
                    .collect(),
            ),
        })
    }
}

impl InnerValue {
    pub fn into_with_recursion_limit(self, recursion_limit: usize) -> Value {
        match self {
            InnerValue::Null => Value::Null,
            InnerValue::Bool(b) => Value::Bool(b),
            InnerValue::Number(n) => Value::Number(n),
            InnerValue::String(s) => Value::String(s),
            InnerValue::Array(a) => Value::Array(
                a.into_iter()
                    .filter(|_| recursion_limit - 1 != 0)
                    .map(|e| {
                        gc_borrow_inner!(e)
                            .clone()
                            .into_with_recursion_limit(recursion_limit - 1)
                    })
                    .collect(),
            ),
            InnerValue::Object(m) => Value::Object(
                m.into_iter()
                    .filter(|_| recursion_limit - 1 != 0)
                    .map(|(k, v)| {
                        (
                            k,
                            gc_borrow_inner!(&v.borrow())
                                .clone()
                                .into_with_recursion_limit(recursion_limit - 1),
                        )
                    })
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

pub unsafe fn collect_cycles_if_needed() {
    let current_time = Instant::now();

    static mut LAST_TIME: Option<Instant> = None;
    if LAST_TIME.is_none() {
        LAST_TIME = Some(current_time);
    }

    if current_time.duration_since(LAST_TIME.unwrap()) >= Duration::from_secs(1)
        && number_of_roots_buffered() >= 1
    {
        LAST_TIME = Some(current_time);

        collect_cycles()
    }
}
