#if defined _grip_included
	#endinput
#endif
#define _grip_included

#pragma reqlib grip
#if !defined AMXMODX_NOAUTOLOAD
	#pragma loadlib grip
#endif


enum GripRequestType {
	GripRequestTypeGet = 0,
	GripRequestTypePost = 1,
	GripRequestTypePut = 2,
	GripRequestTypeDelete = 3
}

enum GripRequestCancellation {
	Invalid_GripRequestCancellation = 0,
}

enum GripRequestOptions {
	Empty_GripRequestOptions = -1,
	Invalid_GripRequestOptions = 0,
}

enum GripBody {
	Empty_GripBody = -1,
	Invalid_GripBody = 0,
}

enum GripJSONValue {
    Invalid_GripJSONValue = 0,
}

enum GripResponseState {
	GripResponseStateCancelled = 1,
	GripResponseStateError = 2,
	GripResponseStateSuccessful = 3,
}

/**
 * Get's state of the response. 
 *
 * @note    		This has nothing to do with HTTP status codes.
 *
 * @param str		Zero terminated string from which body should be created
 *  
 * @return			Returns current response state.
 */
native GripResponseState:grip_get_response_state();

/**
 * Returns whether request exists/active. 
 *
 * @note 			This has nothing to do with HTTP status codes.
 *
 * @param str		Zero terminated string from which body should be created
 *  
 * @return			Returns current response state.
 */
native grip_is_request_active(GripRequestCancellation:request);

/**
 * Creates new body handle from string
 * 
 * @note 			Body should be destroyed with the relevant call.
 *
 * @param str		Zero terminated string from which body should be created
 *  
 * @return			Newly crated body handle
 */
native GripBody:grip_body_from_string(str[]);

/**
 * Destroys body handle
 *
 * @param body		Body to be destroyed
 *  
 * @noreturn
 */
native grip_destroy_body(GripBody:body);

/**
 * Starts sending of the request  
 * @note	The handle should look like:
 * 		public RequestHandler(const userData);
 *
 *
 * @param uri		Request URI. Supports TLS.
 * @param type		Request type which should be sended.
 * @param body		Reqeust body, can be either JSON or plaintext 
 * @param handler	A callback which will be called when request finishes execution
 * @param options	Request options containing HTTP headers, timeout and so on..
 * @param userData 	User data (can be datapack or anything)
 *  
 * @return		Cancellation handle.
 */
native GripRequestCancellation:grip_request(const uri[], GripBody:body, GripRequestType:type, const handler[], GripRequestOptions:options = Empty_GripRequestOptions, const userData = 0);

/**
 * Cancel sending of the request and receiving of response.  
 *
 * @param cancellation	Cancellation handle of the request.
 *  
 * @return		Request handle
 */
native grip_cancel_request(GripRequestCancellation:cancellation);

/**
 * Get current error description. Implementation defined.  
 *
 * @note There are certain AMXX limitations to maximum string sizes.  
 *
 * @param buffer	Output buffer to which description should be written
 * @param buffer_size	Maximum length of the buffer.
 *  
 * @noreturn
 */
native grip_get_error_description(buffer[], buffer_size);

/**
 * Get current response body as string.
 *
 * @note There are certain AMXX limitations to maximum string sizes.  
 *
 * @param buffer	Output buffer to which body should be written
 * @param buffer_size	Maximum length of the buffer.
 *  
 * @noreturn
 */
native grip_get_response_body_string(buffer[], buffer_size);

/**
 * Parse current response body as JSON.
 *
 * @param error_buffer	        Output buffer to which possible error should be written
 * @param error_buffer_size	Maximum length of the buffer.
 *
 *
 * @return                      JSON Value handle. If error occurred, returns invalid handle.
 */
native GripJSONValue:grip_parse_response_body_as_json(error_buffer[], error_buffer_size);

/**
 * Destroy this JSON value
 *
 * @param json_value	JSON Value to be destroyed.
 *
 * @noreturn
 */
native grip_destroy_json_value(GripJSONValue:json_value);

/**
 * Create options with empty headers and some timeout.
 *
 * @note 		Options should be destroyed with the relevant call.
 *
 * @param timeout	timeout -1 to disable, >0 to enable timeout.
 *
 * @return 		Request options handle.
 */
native GripRequestOptions:grip_create_default_options(Float:timeout = -1);

/**
 * Destroy this options
 *
 * @noreturn
 */
native grip_destroy_options(GripRequestOptions:options);

/**
 * Add HTTP Header to this options.
 *.
 * @param options	Options to which header should be added
 * @param headerName 	Header name
 * @param headerValue	Header value
 *
 * @return 		Request options handle.
 */
native grip_options_add_header(GripRequestOptions:options, const headerName[], const headerValue[]);


/**
 * Create options with headers and some timeout.
 *
 * @note 		Options should be destroyed with the relevant call.
 *
 * @param headers	Array of heaer pairs.
 * @param headers_count	Count of the header pairs.
 * @param timeout	timeout -1 to disable, >0 to enable timeout.
 *
 * @return 		Request options handle.
 */
stock grip_create_create_options(const headers[][2][], const headers_count, Float:timeout = -1) {
	new GripRequestOptions:options = grip_create_default_options(timeout);
	
	for(int i = 0; i < headers_count) {
		grip_options_add_header(options, headers[i][0], headers[i][1]);
	}
	
	return options;
}

/* AMXX-Studio Notes - DO NOT MODIFY BELOW HERE
*{\\ rtf1\\ ansi\\ ansicpg1252\\ uc1\\ deff0\\ deflang1033\\ deflangfe1033{\\ fonttbl{\\ f0 Tahoma;}}\n{\\ colortbl;}\n{\\ *\\ generator Wine Riched20 2.0;}\n\\ pard\\ sl-240\\ slmult1 \\ lang1033\\ fs16\\ f0 \\ par\n}
*/
