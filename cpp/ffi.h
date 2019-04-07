#ifndef GRIP_FFI_H
#define GRIP_FFI_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include "amxxmodule.h"

extern "C" {

cell grip_body_from_string(const void *amx, const char *str);

cell grip_cancel_request(const void *amx, cell cancellation);

cell grip_create_default_options(const void *amx, double timeout);

void grip_deinit();

cell grip_destroy_body(const void *amx, cell body);

cell grip_destroy_json_value(const void *amx, cell json_value);

cell grip_destroy_options(const void *amx, cell options_handle);

cell grip_get_error_description(const void *amx, char *buffer, cell size);

cell grip_get_response_body_string(const void *amx, char *buffer, cell size);

cell grip_get_response_state(const void *amx);

cell grip_get_response_status_code(const void *amx);

void grip_init(void (*error_logger)(const void*, const char*), const char *config_file_path);

cell grip_is_request_active(cell request_id);

cell grip_json_array_append_bool(const void *amx, cell array, bool value);

cell grip_json_array_append_float(const void *amx, cell array, float value);

cell grip_json_array_append_null(const void *amx, cell array);

cell grip_json_array_append_number(const void *amx, cell array, cell value);

cell grip_json_array_append_string(const void *amx, cell array, const char *string);

cell grip_json_array_append_value(const void *amx, cell array, cell value);

cell grip_json_array_clear(const void *amx, cell array);

cell grip_json_array_get_bool(const void *amx, cell array, cell index);

cell grip_json_array_get_count(const void *amx, cell array);

cell grip_json_array_get_float(const void *amx, cell array, cell index, float *ret);

cell grip_json_array_get_number(const void *amx, cell array, cell index);

cell grip_json_array_get_string(const void *amx,
                                cell array,
                                cell index,
                                char *buffer,
                                cell buffer_size);

cell grip_json_array_get_value(const void *amx, cell array, cell index);

cell grip_json_array_remove(const void *amx, cell array, cell index);

cell grip_json_array_replace_bool(const void *amx, cell array, cell index, bool value);

cell grip_json_array_replace_float(const void *amx, cell array, cell index, float value);

cell grip_json_array_replace_null(const void *amx, cell array, cell index);

cell grip_json_array_replace_number(const void *amx, cell array, cell index, cell value);

cell grip_json_array_replace_string(const void *amx, cell array, cell index, const char *string);

cell grip_json_array_replace_value(const void *amx, cell array, cell index, cell value);

cell grip_json_equals(const void *amx, cell value1, cell value2);

cell grip_json_get_bool(const void *amx, cell value);

cell grip_json_get_float(const void *amx, cell value, float *ret);

cell grip_json_get_number(const void *amx, cell value);

cell grip_json_get_string(const void *amx, cell value, char *buffer, cell buffer_size);

cell grip_json_get_type(const void *amx, cell value);

cell grip_json_init_array();

cell grip_json_init_bool(bool value);

cell grip_json_init_float(double value);

cell grip_json_init_null();

cell grip_json_init_number(cell value);

cell grip_json_init_object();

cell grip_json_init_string(const void *amx, char *string);

cell grip_json_object_clear(const void *amx, cell object);

cell grip_json_object_get_bool(const void *amx, cell object, const char *name, bool dot_notation);

cell grip_json_object_get_count(const void *amx, cell object);

cell grip_json_object_get_float(const void *amx,
                                cell object,
                                const char *name,
                                bool dot_notation,
                                float *ret);

cell grip_json_object_get_name(const void *amx, cell object, cell index, char *buffer, cell maxlen);

cell grip_json_object_get_number(const void *amx, cell object, const char *name, bool dot_notation);

cell grip_json_object_get_string(const void *amx,
                                 cell object,
                                 const char *name,
                                 char *buffer,
                                 cell maxlen,
                                 bool dot_notation);

cell grip_json_object_get_value(const void *amx, cell object, const char *name, bool dot_notation);

cell grip_json_object_get_value_at(const void *amx, cell object, cell index);

cell grip_json_object_has_value(const void *amx,
                                cell object,
                                const char *name,
                                cell json_type,
                                bool dot_notation);

cell grip_json_object_remove(const void *amx, cell object, const char *name, bool dot_notation);

cell grip_json_object_set_bool(const void *amx,
                               cell object,
                               const char *name,
                               bool value,
                               bool dot_notation);

cell grip_json_object_set_float(const void *amx,
                                cell object,
                                const char *name,
                                float number,
                                bool dot_notation);

cell grip_json_object_set_null(const void *amx, cell object, const char *name, bool dot_notation);

cell grip_json_object_set_number(const void *amx,
                                 cell object,
                                 const char *name,
                                 cell number,
                                 bool dot_notation);

cell grip_json_object_set_string(const void *amx,
                                 cell object,
                                 const char *name,
                                 const char *string,
                                 bool dot_notation);

cell grip_json_object_set_value(const void *amx,
                                cell object,
                                const char *name,
                                cell value,
                                bool dot_notation);

cell grip_json_parse_file(const void *amx, char *file, char *error_buffer, cell error_buffer_size);

cell grip_json_parse_response_body(const void *amx, char *error_buffer, cell error_buffer_size);

cell grip_json_parse_string(const void *amx,
                            char *string,
                            char *error_buffer,
                            cell error_buffer_size);

cell grip_options_add_header(const void *amx,
                             cell options_handle,
                             const char *header_name,
                             const char *header_value);

void grip_process_request();

cell grip_request(const void *amx,
                  cell forward_id,
                  const char *uri,
                  cell body_handle,
                  cell request_type,
                  void (*handler)(cell forward_handle, cell user_data),
                  cell options_handle,
                  cell user_data);

} // extern "C"

#endif // GRIP_FFI_H
