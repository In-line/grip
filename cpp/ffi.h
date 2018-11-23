//
// Created by alik on 21.11.18.
//

#ifndef RESTRY_FFI_H
#define RESTRY_FFI_H


extern "C" {
//    typedef (*void) RestryRequest;
//    RestryRequest restry_create_post_request();
//    RestryRequest destroy_request();

    void restry_init();
    void restry_deinit();
}

#endif //RESTRY_FFI_H
