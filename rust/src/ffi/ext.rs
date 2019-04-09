/*
 * gRIP
 * Copyright (c) 2018 Alik Aslanyan <cplusplus256@gmail.com>
 *
 *
 *    This program is free software; you can redistribute it and/or modify it
 *    under the terms of the GNU General Public License as published by the
 *    Free Software Foundation; either version 3 of the License, or (at
 *    your option) any later version.
 *
 *    This program is distributed in the hope that it will be useful, but
 *    WITHOUT ANY WARRANTY; without even the implied warranty of
 *    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 *    General Public License for more details.
 *
 *    You should have received a copy of the GNU General Public License
 *    along with this program; if not, write to the Free Software Foundation,
 *    Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 *    In addition, as a special exception, the author gives permission to
 *    link the code of this program with the Half-Life Game Engine ("HL
 *    Engine") and Modified Game Libraries ("MODs") developed by Valve,
 *    L.L.C ("Valve").  You must obey the GNU General Public License in all
 *    respects for all of the code used other than the HL Engine and MODs
 *    from Valve.  If you modify this file, you may extend this exception
 *    to your version of the file, but you are not obligated to do so.  If
 *    you do not wish to do so, delete this exception statement from your
 *    version.
 *
 */

use crate::errors::*;
use crate::gc_json::*;
use std::ffi::CStr;
use std::os::raw::c_char;

pub trait ResultFFIExt<T> {
    fn get_value(self) -> std::result::Result<T, String>;
}

impl<T> ResultFFIExt<T> for Result<T> {
    fn get_value(self) -> std::result::Result<T, String> {
        use error_chain::ChainedError;
        self.map_err(|e| format!("{}", e.display_chain()))
    }
}

impl<T> ResultFFIExt<T> for Option<T> {
    fn get_value(self) -> std::result::Result<T, String> {
        self.ok_or_else(|| "Got empty option".to_owned())
    }
}

macro_rules! try_and_log_ffi {
    ($amx:expr, $expr:expr, $error_logger:expr) => {
        match $expr.get_value() {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err(err) => {
                ($error_logger)($amx, err);
                return 0;
            }
        }
    };

    ($amx:expr, $expr:expr) => {
        try_and_log_ffi!($amx, $expr, |amx, err| {
            (get_module().error_logger)(amx, format!("{}\0", err).as_ptr() as *const c_char);
        });
    };
}

pub fn ptr_to_option<T>(ptr: *const T) -> Option<*const T> {
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

pub unsafe fn str_from_ptr<'a>(value: *const c_char) -> Result<&'a str> {
    CStr::from_ptr(value)
        .to_str()
        .chain_err(|| "Can't create string from raw pointer.")
}

macro_rules! try_as_usize {
    ($amx:expr, $size:expr, $error_logger:expr) => {
        try_and_log_ffi!(
            $amx,
            if $size >= 0 {
                Ok($size as usize)
            } else {
                Err(ffi_error(format!(
                    "Index/Size {} should be greater or equal to zero.",
                    $size
                )))
            },
            $error_logger
        )
    };

    ($amx:expr, $size:expr) => {
        try_as_usize!($amx, $size, |amx, err| {
            (get_module().error_logger)(amx, format!("{}\0", err).as_ptr() as *const c_char);
        })
    };
}

macro_rules! try_to_copy_unsafe_string {
    ($amx:expr, $dest:expr, $source:expr, $charsmax:expr, $error_logger:expr) => {{
        crate::ffi::strlcpy::strlcpy(
            $dest,
            format!("{}\0", $source).as_ptr() as *const c_char,
            try_as_usize!($amx, $charsmax, $error_logger) + 1,
        ) as Cell
    }};

    ($amx:expr, $dest:expr, $source:expr, $size:expr) => {
        try_to_copy_unsafe_string!($amx, $dest, $source, $size, |amx, err| {
            (get_module().error_logger)(amx, format!("{}\0", err).as_ptr() as *const c_char);
        })
    };
}

macro_rules! unconditionally_log_error {
    ($amx:expr, $err:expr, $error_logger:expr) => {
        try_and_log_ffi!($amx, Err($err), $error_logger)
    };

    ($amx:expr, $err:expr) => {
        unconditionally_log_error!($amx, $err, |amx, err| {
            (get_module().error_logger)(amx, format!("{}\0", err).as_ptr() as *const c_char);
        })
    };
}

macro_rules! try_to_get_json_value_gc {
    ($amx:expr, $value:expr) => {{
        let value: &GCValue = try_and_log_ffi!(
            $amx,
            get_module_mut()
                .json_handles
                .get_with_id($value)
                .chain_err(|| ffi_error(format!("Invalid JSON value handle {}", $value)))
        );

        value
    }};
}
macro_rules! try_to_get_json_value {
    ($amx:expr, $value:expr) => {{
        gc_borrow_inner!(try_to_get_json_value_gc!($amx, $value))
    }};
}

macro_rules! try_to_get_json_value_gc_mut {
    ($amx:expr, $value:expr) => {{
        let value: &mut GCValue = try_and_log_ffi!(
            $amx,
            get_module_mut()
                .json_handles
                .get_mut_with_id($value)
                .chain_err(|| ffi_error(format!("Invalid JSON value handle {}", $value)))
        );

        value
    }};
}

macro_rules! try_to_get_json_value_mut {
    ($amx:expr, $value:expr) => {{
        gc_borrow_inner_mut!(try_to_get_json_value_gc_mut!($amx, $value)) as &mut InnerValue
    }};
}

macro_rules! try_to_get_json_object_value_rc_gc {
    ($amx:expr, $object:expr, $name:expr, $dot_notation:expr) => {
        try_and_log_ffi!(
            $amx,
            try_to_get_json_value_gc!($amx, $object).index_selective_safe_rc_gc(
                try_and_log_ffi!($amx, str_from_ptr($name)),
                $dot_notation
            )
        )
    };
}

macro_rules! try_to_get_json_object_value_or_insert_rc_gc {
    ($amx:expr, $object:expr, $name:expr, $dot_notation:expr) => {
        try_and_log_ffi!(
            $amx,
            try_to_get_json_value_gc!($amx, $object).index_selective_safe_or_insert_rc_gc(
                try_and_log_ffi!($amx, str_from_ptr($name)),
                $dot_notation,
                Some(InnerValue::Null)
            )
        )
    };
}

macro_rules! try_to_get_json_object_value_gc {
    ($amx:expr, $object:expr, $name:expr, $dot_notation:expr) => {{
        try_to_get_json_object_value_rc_gc!($amx, $object, $name, $dot_notation).borrow()
    }};
}

macro_rules! try_to_get_json_object_value_gc_or_insert_mut {
    ($amx:expr, $object:expr, $name:expr, $dot_notation:expr) => {
        (&*try_to_get_json_object_value_or_insert_rc_gc!($amx, $object, $name, $dot_notation)
            as &RefCell<GCValue>)
            .borrow_mut()
    };
}

macro_rules! try_to_get_json_object_value {
    ($amx:expr, $object:expr, $name:expr, $dot_notation:expr) => {{
        gc_borrow_inner!(try_to_get_json_object_value_gc!(
            $amx,
            $object,
            $name,
            $dot_notation
        ))
    }};
}

macro_rules! try_to_get_json_object_value_mut {
    ($amx:expr, $object:expr, $name:expr, $dot_notation:expr) => {{
        gc_borrow_inner_mut!(try_to_get_json_object_value_gc_or_insert_mut!(
            $amx,
            $object,
            $name,
            $dot_notation
        ))
    }};
}

macro_rules! try_to_get_json_array {
    ($amx:expr, $array:expr) => {
        match &try_to_get_json_value!($amx, $array) {
            InnerValue::Array(vec) => vec,
            v => unconditionally_log_error!(
                $amx,
                ffi_error(format!("JSON Handle is not array. {:?}", v))
            ),
        }
    };
}

macro_rules! try_to_get_json_array_mut {
    ($amx:expr, $array:expr) => {
        match &mut try_to_get_json_value_mut!($amx, $array) {
            InnerValue::Array(vec) => vec,
            v => unconditionally_log_error!(
                $amx,
                ffi_error(format!("JSON Handle is not array. {:?}", v))
            ),
        }
    };
}

macro_rules! try_to_get_json_array_value {
    ($amx:expr, $array:expr, $index:expr) => {
        try_and_log_ffi!(
            $amx,
            try_to_get_json_array!($amx, $array).get(try_as_usize!($amx, $index))
        )
    };
}

macro_rules! try_to_get_json_array_value_mut {
    ($amx:expr, $array:expr, $index:expr) => {
        try_and_log_ffi!(
            $amx,
            try_to_get_json_array_mut!($amx, $array).get_mut(try_as_usize!($amx, $index))
        )
    };
}

use core::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

pub trait ValueExt<'a> {
    fn dot_index_safe_rc_gc(&'a self, name: &str) -> Result<Rc<RefCell<GCValue>>>;
    fn dot_index_safe_or_insert_rc_gc(
        &'a self,
        name: &str,
        default: Option<InnerValue>,
    ) -> Result<Rc<RefCell<GCValue>>>;
    fn index_selective_safe_rc_gc(
        &'a self,
        name: &str,
        dot_notation: bool,
    ) -> Result<Rc<RefCell<GCValue>>>;

    fn index_selective_safe_or_insert_rc_gc(
        &'a self,
        name: &str,
        dot_notation: bool,
        default: Option<InnerValue>,
    ) -> Result<Rc<RefCell<GCValue>>>;
}

impl<'a> ValueExt<'a> for GCValue {
    fn dot_index_safe_rc_gc(&'a self, name: &str) -> Result<Rc<RefCell<GCValue>>> {
        let mut it: Option<Rc<RefCell<GCValue>>> = None;
        for element in name.split('.') {
            if element.is_empty() {
                bail!("Double/Empty separator in `{}`", name);
            }

            // Same as bounds checked index.
            if let Some(it_raw) = it {
                let it_raw: &RefCell<_> = it_raw.borrow();
                it = Some(
                    it_raw
                        .borrow()
                        .index_selective_safe_rc_gc(element, false)?
                        .clone(),
                );
            } else {
                it = Some(self.index_selective_safe_rc_gc(element, false)?.clone());
            }
        }

        Ok(it.chain_err(|| "Name is invalid")?)
    }

    fn dot_index_safe_or_insert_rc_gc(
        &'a self,
        name: &str,
        default: Option<InnerValue>,
    ) -> Result<Rc<RefCell<GCValue>>> {
        let mut it: Option<Rc<RefCell<GCValue>>> = None;
        let names = name.split('.').collect::<Vec<&str>>();
        let len = names.len();
        for (default_to_insert, element) in {
            names.into_iter().enumerate().map(|(index, string)| {
                (
                    if index == len - 1 {
                        default.clone()
                    } else {
                        None
                    },
                    string,
                )
            })
        } {
            if element.is_empty() {
                bail!("Double/Empty separator in `{}`", name);
            }

            // Same as bounds checked index.
            if let Some(it_raw) = it {
                let it_raw: &RefCell<_> = it_raw.borrow();
                it = Some(
                    it_raw
                        .borrow_mut()
                        .index_selective_safe_or_insert_rc_gc(element, false, default_to_insert)?
                        .clone(),
                );
            } else {
                it = Some(
                    self.index_selective_safe_or_insert_rc_gc(element, false, default_to_insert)?
                        .clone(),
                );
            }
        }

        Ok(it.chain_err(|| "Name is invalid")?)
    }

    fn index_selective_safe_or_insert_rc_gc(
        &'a self,
        name: &str,
        dot_notation: bool,
        default: Option<InnerValue>,
    ) -> Result<Rc<RefCell<GCValue>>> {
        if dot_notation {
            self.dot_index_safe_or_insert_rc_gc(name, default)
        } else {
            match &mut self.borrow_inner_ref_mut() as &mut InnerValue {
                InnerValue::Object(m) => match {
                    match m.get_mut(name) {
                        None => {
                            if let Some(default) = default {
                                m.insert(
                                    name.to_owned(),
                                    Rc::new(RefCell::new(GCValue::new(default))),
                                );
                                m.get_mut(name)
                            } else {
                                None
                            }
                        }
                        v => v,
                    }
                } {
                    Some(target) => Ok(target.clone()),
                    None => bail!(
                        "Can't index json using `{}`, because json doesn't contain it",
                        name
                    ),
                },
                _ => bail!(
                    "Can't index json using `{}` json stops is not object.",
                    name
                ),
            }
        }
    }

    fn index_selective_safe_rc_gc(
        &'a self,
        name: &str,
        dot_notation: bool,
    ) -> Result<Rc<RefCell<GCValue>>> {
        if dot_notation {
            self.dot_index_safe_rc_gc(name)
        } else {
            match &self.borrow_inner_ref() as &InnerValue {
                InnerValue::Object(m) => match m.get(name) {
                    Some(target) => Ok(target.clone()),
                    None => bail!(
                        "Can't index json using `{}`, because json doesn't contain it",
                        name
                    ),
                },
                _ => bail!(
                    "Can't index json using `{}` json stops is not object.",
                    name
                ),
            }
        }
    }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::Cell;
    use libc::c_char;
    use serde_json::{json, Value};
    use std::cell::Ref;

    unsafe fn copy_unsafe_string(size: isize) -> Cell {
        let mut s: [c_char; 2] = [0; 2];

        let status =
            try_to_copy_unsafe_string!(123 as *mut c_char, s.as_mut_ptr(), "1", size, |amx, _| {
                assert!(amx == 123 as *mut c_char);
            });

        assert_eq!(s, ['1' as c_char, '\0' as c_char]);

        status
    }

    #[test]
    fn copy_unsafe_string_test() {
        unsafe {
            assert_eq!(copy_unsafe_string(-1), 0);
            assert_eq!(copy_unsafe_string(2), 1);
        }
    }

    #[test]
    fn dot_index_safe_rc_gc() {
        let json: GCValue = gc_json!({
            "a": {
                "b": 123
            }
        });

        fn gc_to_json(v: Rc<RefCell<GCValue>>) -> Value {
            let value: &RefCell<_> = v.borrow();
            (*gc_borrow_inner!(value.borrow()))
                .clone()
                .into_with_recursion_limit(2)
        }

        assert_eq!(
            gc_to_json(json.dot_index_safe_rc_gc("a.b").unwrap())
                .as_u64()
                .unwrap(),
            123
        );
        assert!(json.dot_index_safe_rc_gc("a.b.c").is_err());
        assert!(json.dot_index_safe_rc_gc("a..").is_err());
        assert!(gc_to_json(json.dot_index_safe_rc_gc("a").unwrap()).is_object());

        assert_eq!(
            gc_to_json(json.index_selective_safe_rc_gc("a.b", true).unwrap())
                .as_u64()
                .unwrap(),
            123
        );
        assert!(json.index_selective_safe_rc_gc("a.b.c", true).is_err());
        assert!(json.index_selective_safe_rc_gc("a..", true).is_err());
        assert!(gc_to_json(json.index_selective_safe_rc_gc("a", true).unwrap()).is_object());

        assert!(gc_to_json(json.index_selective_safe_rc_gc("a", false).unwrap()).is_object());
        assert!(json.index_selective_safe_rc_gc("a.b.c", false).is_err());

        assert_eq!(
            gc_to_json(
                json.index_selective_safe_or_insert_rc_gc("a.b", true, None)
                    .unwrap()
            )
            .as_u64()
            .unwrap(),
            123
        );
        assert!(json
            .index_selective_safe_or_insert_rc_gc("a.b.c", true, None)
            .is_err());
        assert!(json
            .index_selective_safe_or_insert_rc_gc("a..", true, None)
            .is_err());
        assert!(gc_to_json(
            json.index_selective_safe_or_insert_rc_gc("a", true, None)
                .unwrap()
        )
        .is_object());

        assert!(gc_to_json(
            json.index_selective_safe_or_insert_rc_gc("a", false, None)
                .unwrap()
        )
        .is_object());
        assert!(json
            .index_selective_safe_or_insert_rc_gc("a.b.c", false, None)
            .is_err());

        assert!(json
            .index_selective_safe_or_insert_rc_gc("a.d.d", false, None)
            .is_err());

        assert!(gc_to_json(
            json.index_selective_safe_or_insert_rc_gc("a.d", true, Some(InnerValue::Null))
                .unwrap()
        )
        .is_null());

        assert!(json
            .index_selective_safe_or_insert_rc_gc("a.d.d", false, None)
            .is_err());
    }

}
