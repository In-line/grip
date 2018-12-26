//
// Created by alik on 21.11.18.
//

#ifndef RESTRY_FFI_H
#define RESTRY_FFI_H

#include "amxxmodule.h"

extern "C" {
    typedef void (*GripErrorLogger)(const void* amx, const char* str);

    void grip_init(GripErrorLogger logger, const char* config_file_path);
    void grip_deinit();
    void grip_process_request();

    cell grip_body_from_string(const void* amx, const char* str);
    cell grip_destroy_body(const void* amx, cell body);

    typedef void (*GripHandler)(cell, cell, cell);
    cell grip_request(const void* amx, cell forward_id, const char *uri, cell body_handle, cell request_type, GripHandler handler, cell user_data);

    cell grip_cancel_request(const void* amx, cell cancellation_id);
    cell grip_get_response_state(const void* amx);
    cell grip_is_request_active(cell request_id);

    cell grip_get_error_description(const void* amx, char *buffer, cell buffer_size);
    cell grip_get_response_body_string(const void* amx, char *buffer, cell buffer_size);

    cell grip_parse_response_body_as_json(const void* amx, char *error_buffer, cell error_buffer_size);

    cell grip_destroy_json_value(const void* amx, cell json_value);
}
#endif //RESTRY_FFI_H
