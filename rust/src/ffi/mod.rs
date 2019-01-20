extern crate ini;
extern crate libc;

use self::ini::Ini;

#[macro_use]
mod ext;

use crate::ffi::ext::*;

use self::libc::{c_char, c_void};

use std::ffi::CStr;

use crate::errors::*;

type Cell = isize;

static INVALID_CELL: Cell = 0;
use crate::networking_queue::{
    Queue, RequestBuilder, RequestCancellation, RequestOptions, RequestType, Response,
};
use std::prelude::v1::Vec;

use crate::cell_map::CellMap;

struct ModuleStorage {
    pub global_queue: Queue,
    pub current_response: Option<Result<Response>>,
    pub bodies_handles: CellMap<Vec<u8>>,
    pub cancellations_handles: CellMap<RequestCancellation>,
    pub json_handles: CellMap<serde_json::Value>,
    pub options_handles: CellMap<RequestOptions>,
    pub error_logger: extern "C" fn(*const c_void, *const c_char),
    pub callbacks_per_frame: usize,
    pub microseconds_delay_between_attempts: usize,
}

static mut MODULE: Option<ModuleStorage> = None;

#[no_mangle]
pub unsafe extern "C" fn grip_init(
    error_logger: extern "C" fn(*const c_void, *const c_char),
    config_file_path: *const c_char,
) {
    let ini = Ini::load_from_file(CStr::from_ptr(config_file_path).to_str().unwrap())
        .map_err(|e| {
            println!(
                "Error: Can't parse/open grip config. Examine carefully ini parser log message\n{}",
                e
            );
            e
        })
        .unwrap();

    let dns_section = ini
        .section(Some("dns".to_owned()))
        .or_else(|| {
            println!("Missing [dns] section in the grip.ini config");
            None
        })
        .unwrap();

    let queue_section = ini
        .section(Some("queue".to_owned()))
        .or_else(|| {
            println!("Error: Missing [queue] section in the grip.ini config");
            None
        })
        .unwrap();

    MODULE = Some(ModuleStorage {
        global_queue: Queue::new(
            dns_section
                .get("number-of-dns-threads")
                .or_else(|| {
                    println!(
                        "Error: Missing \"dns.number-of-dns-threads\" key in the grip.ini config"
                    );
                    None
                })
                .unwrap()
                .parse()
                .unwrap(),
        ),
        cancellations_handles: CellMap::new(),
        current_response: None,
        bodies_handles: CellMap::new(),
        json_handles: CellMap::new(),
        options_handles: CellMap::new(),
        error_logger,
        callbacks_per_frame: {
            queue_section
                .get("callbacks-per-frame")
                .or_else(|| {
                    println!(
                        "Error: Missing \"queue.callbacks-per-frame\" key in the grip.ini config"
                    );
                    None
                })
                .unwrap()
                .parse()
                .unwrap()
        },
        microseconds_delay_between_attempts: {
            queue_section
                .get("microseconds-delay-between-attempts")
                .or_else(|| {
                    println!("Error: Missing \"queue.microseconds-delay-between-attempts\" key in the grip.ini config");
                    None
                }).unwrap()
                .parse()
                .unwrap()
        },
    });
}

unsafe fn get_module() -> &'static ModuleStorage {
    MODULE.as_ref().unwrap()
}

unsafe fn get_module_mut() -> &'static mut ModuleStorage {
    MODULE.as_mut().unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn grip_deinit() {
    if MODULE.is_some() {
        get_module_mut().cancellations_handles.clear(); // Cancel all operations, before queue stopped.
    }
    MODULE = None;
}

#[no_mangle]
pub unsafe extern "C" fn grip_destroy_body(amx: *const c_void, body: Cell) -> Cell {
    try_and_log_ffi!(
        amx,
        get_module_mut()
            .bodies_handles
            .remove_with_id(body)
            .chain_err(|| ffi_error(format!("Invalid body handle {}", body)))
    );

    1
}

#[no_mangle]
pub unsafe extern "C" fn grip_body_from_string(amx: *const c_void, str: *const c_char) -> Cell {
    get_module_mut().bodies_handles.insert_with_unique_id(
        CStr::from_ptr(try_and_log_ffi!(
            amx,
            handle_null_ptr(str).chain_err(|| ffi_error("Invalid URI."))
        ))
        .to_bytes()
        .iter()
        .cloned()
        .collect(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn grip_request(
    amx: *const c_void,
    forward_id: Cell,
    uri: *const c_char,
    body_handle: Cell,
    request_type: Cell,
    handler: Option<extern "C" fn(forward_handle: Cell, user_data: Cell) -> c_void>,
    options_handle: Cell,
    user_data: Cell,
) -> Cell {
    let request_type = try_and_log_ffi!(
        amx,
        match request_type {
            0 => Ok(RequestType::Get),
            1 => Ok(RequestType::Post),
            2 => Ok(RequestType::Put),
            3 => Ok(RequestType::Delete),
            _ => Err(ErrorKind::FFIError(format!("Invalid request type {}", request_type)).into()),
        }
    );

    let uri = try_and_log_ffi!(
        amx,
        CStr::from_ptr(try_and_log_ffi!(
            amx,
            handle_null_ptr(uri).chain_err(|| ffi_error("Invalid URI."))
        ))
        .to_str()
        .map_err(|_| ffi_error("URI is not UTF-8"))
    );

    let body = try_and_log_ffi!(
        amx,
        get_module()
            .bodies_handles
            .get_with_id(body_handle)
            .or_else(|| if body_handle == -1 {
                lazy_static! {
                    static ref EMPTY_VEC: Vec<u8> = vec![];
                }
                Some(&EMPTY_VEC)
            } else {
                None
            })
            .chain_err(|| ffi_error(format!("Invalid body handle: {}", body_handle)))
    );

    let options = try_and_log_ffi!(
        amx,
        get_module()
            .options_handles
            .get_with_id(options_handle)
            .or_else(|| if options_handle == -1 {
                lazy_static! {
                    static ref EMPTY_OPTIONS: RequestOptions = RequestOptions::default();
                }
                Some(&EMPTY_OPTIONS)
            } else {
                None
            })
            .chain_err(|| ffi_error(format!("Invalid options handle: {}", options_handle)))
    );

    // TODO: JSON, Headers, Timeout.

    let next_cancellation_id = get_module().cancellations_handles.peek_id();
    let cancellation = get_module_mut().global_queue.send_request(
        RequestBuilder::default()
            .http_type(request_type)
            .body(body.clone())
            .uri(try_and_log_ffi!(
                amx,
                uri.parse()
                    .chain_err(|| ffi_error(format!("URI parsing error: {}", uri)))
            ))
            .options(options.clone())
            .build()
            .unwrap(),
        move |response| {
            get_module_mut().current_response = Some(response);

            handler.unwrap()(forward_id, user_data);

            get_module_mut()
                .cancellations_handles
                .remove_with_id(next_cancellation_id);

            get_module_mut().current_response = None;
        },
    );

    get_module_mut()
        .cancellations_handles
        .insert_with_unique_id(cancellation)
}

//cell grip_cancel_request(const void* amx, cell cancellation);
#[no_mangle]
pub unsafe extern "C" fn grip_cancel_request(amx: *const c_void, cancellation: Cell) -> Cell {
    try_and_log_ffi!(
        amx,
        get_module_mut()
            .cancellations_handles
            .remove_with_id(cancellation)
            .chain_err(|| ffi_error(format!(
                "Cancellation with the id {} doesn't exist.",
                cancellation
            )))
    );

    1
}

#[no_mangle]
pub unsafe extern "C" fn grip_get_response_state(amx: *const c_void) -> Cell {
    match try_and_log_ffi!(
        amx,
        get_module()
            .current_response
            .as_ref()
            .chain_err(|| ffi_error("Response state can only be received in the request callback"))
    ) {
        Err(e) => match e.kind() {
            crate::errors::ErrorKind::RequestCancelled => 1,
            crate::errors::ErrorKind::RequestTimeout => 4,
            _ => 2,
        },
        Ok(_) => 3,
    }
}

#[no_mangle]
pub unsafe extern "C" fn grip_is_request_active(request_id: Cell) -> Cell {
    if get_module()
        .cancellations_handles
        .get_with_id(request_id)
        .is_some()
    {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn grip_get_error_description(
    amx: *const c_void,
    buffer: *mut c_char,
    size: Cell,
) -> Cell {
    if let Err(e) = try_and_log_ffi!(
        amx,
        get_module()
            .current_response
            .as_ref()
            .chain_err(|| ffi_error("No active response at this time"))
    ) {
        try_and_log_ffi!(
            amx,
            match e.kind() {
                ErrorKind::RequestCancelled => Err(ErrorKind::RequestCancelled.into()),
                _ => Ok(()),
            }
        );

        use error_chain::ChainedError;
        libc::strncpy(
            buffer,
            format!("{}\0", e.display_chain()).as_ptr() as *const c_char,
            try_and_log_ffi!(
                amx,
                if size >= 0 {
                    Ok(size as usize)
                } else {
                    Err(ffi_error(format!(
                        "Size {} should be greater or equal to zero.",
                        size
                    )))
                },
            ),
        );

        *buffer.offset(size) = '\0' as i8;
    } else {
        try_and_log_ffi!(amx, Err(ffi_error("No error for this response.")));
    }

    1
}

// TODO: Remove copy paste
#[no_mangle]
pub unsafe extern "C" fn grip_get_response_body_string(
    amx: *const c_void,
    buffer: *mut c_char,
    size: Cell,
) -> Cell {
    if let Ok(response) = try_and_log_ffi!(
        amx,
        get_module()
            .current_response
            .as_ref()
            .chain_err(|| ffi_error("No active response at this time"))
    ) {
        libc::strncpy(
            buffer,
            format!(
                "{}\0",
                try_and_log_ffi!(
                    amx,
                    std::str::from_utf8(&response.body[..])
                        .chain_err(|| ffi_error("Unable to parse UTF-8"))
                )
            )
            .as_ptr() as *const c_char,
            try_and_log_ffi!(
                amx,
                if size >= 0 {
                    Ok(size as usize)
                } else {
                    Err(ffi_error(format!(
                        "Size {} should be greater or equal to zero.",
                        size
                    )))
                },
            ),
        );

        *buffer.offset(size) = '\0' as i8;
    } else {
        try_and_log_ffi!(amx, Err(ffi_error("Error/Cancellation/Timeout occurred for this response.")));
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn grip_get_response_status_code(
    amx: *const c_void,
) -> Cell {
    if let Ok(response) = try_and_log_ffi!(
        amx,
        get_module()
            .current_response
            .as_ref()
            .chain_err(|| ffi_error("No active response at this time"))
    ) {
        response.status_code.as_u16() as Cell
    } else {
        try_and_log_ffi!(amx, Err(ffi_error("Error/Cancellation/Timeout occurred for this response.")));
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn grip_parse_response_body_as_json(
    amx: *const c_void,
    error_buffer: *mut c_char,
    error_buffer_size: Cell,
) -> Cell {
    if let Ok(response) = try_and_log_ffi!(
        amx,
        get_module()
            .current_response
            .as_ref()
            .chain_err(|| ffi_error("No active response at this time"))
    ) {
        let value: Result<serde_json::Value> =
            serde_json::from_slice(&response.body[..]).map_err(|e| ErrorKind::JSONError(e).into());

        match value {
            Ok(value) => get_module_mut().json_handles.insert_with_unique_id(value),
            Err(error) => {
                use error_chain::ChainedError;
                libc::strncpy(
                    error_buffer,
                    format!("{}\0", error.display_chain()).as_ptr() as *const c_char,
                    try_and_log_ffi!(
                        amx,
                        if error_buffer_size >= 0 {
                            Ok(error_buffer_size as usize)
                        } else {
                            Err(ffi_error(format!(
                                "Size {} should be greater or equal to zero.",
                                error_buffer_size
                            )))
                        },
                    ),
                );

                0
            }
        }
    } else {
        try_and_log_ffi!(amx, Err(ffi_error("Error/Cancellation/Timeout occurred for this response.")));
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn grip_destroy_json_value(amx: *const c_void, json_value: Cell) -> Cell {
    try_and_log_ffi!(
        amx,
        get_module_mut()
            .json_handles
            .remove_with_id(json_value)
            .chain_err(|| ffi_error(format!("Invalid json value handle {}", json_value)))
    );

    1
}

#[no_mangle]
pub unsafe extern "C" fn grip_create_default_options(amx: *const c_void, timeout: f64) -> Cell {
    use float_cmp::ApproxEq;

    get_module_mut()
        .options_handles
        .insert_with_unique_id(RequestOptions::new(
            hyper::HeaderMap::default(),
            try_and_log_ffi!(
                amx,
                if timeout.approx_eq(&-1.0, std::f64::EPSILON, 2) {
                    Ok(None)
                } else if timeout >= 0.0 {
                    Ok(Some(std::time::Duration::from_millis(
                        (timeout * 1000.0) as u64,
                    )))
                } else {
                    Err(ffi_error(format!("Invalid timeout: {}", timeout)))
                }
            ),
        ))
}

#[no_mangle]
pub unsafe extern "C" fn grip_destroy_options(amx: *const c_void, options_handle: Cell) -> Cell {
    try_and_log_ffi!(
        amx,
        get_module_mut()
            .options_handles
            .remove_with_id(options_handle)
            .chain_err(|| ffi_error(format!("Invalid options handle {}", options_handle)))
    );

    1
}

#[no_mangle]
pub unsafe extern "C" fn grip_options_add_header(
    amx: *const c_void,
    options_handle: Cell,
    header_name: *const c_char,
    header_value: *const c_char,
) -> Cell {
    let option = try_and_log_ffi!(
        amx,
        get_module_mut()
            .options_handles
            .get_mut_with_id(options_handle)
            .chain_err(|| ffi_error(format!("Invalid options handle: {}", options_handle))),
    );

    let header_name = try_and_log_ffi!(
        amx,
        CStr::from_ptr(header_name)
            .to_str()
            .chain_err(|| ffi_error("Invalid header name. Can't create UTF-8 string"))
    );

    let header_value = try_and_log_ffi!(
        amx,
        CStr::from_ptr(header_value)
            .to_str()
            .chain_err(|| ffi_error("Invalid header value. Can't create UTF-8 string"))
    );

    let header_value = try_and_log_ffi!(
        amx,
        hyper::header::HeaderValue::from_str(header_value)
        .chain_err(|| ffi_error(format!("Header value contains invalid byte sequences or was rejected by Hyper HTTP implementation: {}", header_value)))
    );

    option.headers.insert(header_name, header_value);

    1
}

#[no_mangle]
pub unsafe extern "C" fn grip_process_request() {
    let multiplier = std::cmp::min(
        get_module().global_queue.number_of_pending_requests() / 500,
        1,
    );
    if multiplier > 1 {
        println!("[gRIP] Warning: More than 500 requests are pending.. Fastening execution {} times to compensate that", multiplier);
    }

    get_module_mut().global_queue.execute_queue_with_limit(
        get_module().callbacks_per_frame * multiplier,
        std::time::Duration::from_micros(get_module().microseconds_delay_between_attempts as u64),
    );
}
