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
    
// void handler(cell forward_handle, cell response_handle, const cell *user_data, cell user_data_size);
    typedef void (*GripHandler)(cell, cell, const cell*, cell);
    cell grip_request(const void* amx, cell forward_id, const char *uri, cell request_type, cell body_handle, GripHandler handler,  const cell *user_data, cell user_data_size);
}
#endif //RESTRY_FFI_H
