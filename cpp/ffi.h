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
    cell grip_request(const void* amx, cell forward_id, const char *uri, cell body_handle, cell request_type, GripHandler handler, cell options_handle, cell user_data);

    cell grip_cancel_request(const void* amx, cell cancellation_id);
    cell grip_get_response_state(const void* amx);
    cell grip_is_request_active(cell request_id);

    cell grip_get_error_description(const void* amx, char *buffer, cell buffer_size);
    cell grip_get_response_body_string(const void* amx, char *buffer, cell buffer_size);



    cell grip_destroy_json_value(const void* amx, cell json_value);

    cell grip_create_default_options(const void *amx, double timeout);

    cell grip_destroy_options(const void* amx, cell options_handle);

    cell grip_options_add_header(const void* amx, cell options_handle, const char* header_name, const char* header_value);

    cell grip_get_response_status_code(const void* amx);

    cell grip_json_parse_response_body(const void* amx, char *error_buffer, cell error_buffer_size);
    cell grip_json_parse_string(const void* amx, const char *string, char *error_buffer, cell error_buffer_size);
    cell grip_json_parse_file(const void* amx, const char *file, char *error_buffer, cell error_buffer_size);

    cell grip_json_equals(const void* amx, cell value1, cell value2);
}
#endif //RESTRY_FFI_H
