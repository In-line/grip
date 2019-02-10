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

use crate::ffi::*;

pub trait ResultFFIExt<T> {
    unsafe fn handle_ffi_error(self, amx: *const c_void) -> std::result::Result<T, Cell>;
}

impl<T> ResultFFIExt<T> for Result<T> {
    unsafe fn handle_ffi_error(self, amx: *const c_void) -> std::result::Result<T, Cell> {
        self.map_err(|err| {
            use error_chain::ChainedError;
            // TODO: More fancy and better formatted error message
            (get_module().error_logger)(
                amx,
                format!("{}\0", err.display_chain()).as_ptr() as *const c_char,
            );
            INVALID_CELL
        })
    }
}

impl<T> ResultFFIExt<T> for Option<T> {
    unsafe fn handle_ffi_error(self, amx: *const c_void) -> std::result::Result<T, Cell> {
        self.ok_or(INVALID_CELL).map_err(|_| {
            (get_module().error_logger)(amx, "Got null pointer\0".as_ptr() as *const c_char);
            INVALID_CELL
        })
    }
}

macro_rules! try_and_log_ffi {
    ($amx:expr, $expr:expr) => {
        match $expr.handle_ffi_error($amx) {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err(err) => return err,
        }
    };
    ($amx:expr, $expr:expr,) => {
        try_and_log_ffi!($amx, $expr)
    };
}

pub fn handle_null_ptr<T>(ptr: *const T) -> Option<*const T> {
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}
