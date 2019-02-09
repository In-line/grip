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
#![recursion_limit = "1024"]
extern crate bytes;
extern crate crossbeam_channel;
extern crate futures;
extern crate reqwest;
extern crate tokio;

#[macro_use]
extern crate log;

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clone_all;

#[macro_use]
extern crate lazy_static;

extern crate serde_json;

extern crate float_cmp;

extern crate itertools;

mod errors {
    error_chain! {
        errors {
            FFIError(t: String) {
                display("FFI Error: {}", t)
            }
            RequestCancelled {
                display("Request was cancelled")
            }
            RequestTimeout {
                display("Request timeout")
            }
        }

        foreign_links {
            CrossBeamError(::crossbeam_channel::TryRecvError);
            HTTPError(::reqwest::Error);
            JSONError(::serde_json::Error);
        }
    }

    pub fn ffi_error<T: Into<String>>(t: T) -> Error {
        ErrorKind::FFIError(t.into()).into()
    }
}

pub mod cell_map;
pub mod ffi;
pub mod networking_queue;
