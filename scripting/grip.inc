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

enum GripRequestOptionsHandle {
	Invalid_GripRequestOptionsHandle = 0,
}

enum GripBodyHandle {
	Empty_GripBodyHandle = -1,
	Invalid_GripBodyHandle = 0,
}

enum GripResponseState {
	GripResponseStateCancelled = 1,
	GripResponseStateError = 2,
	GripResponseStateSuccessful = 3,
}

/**
 * Get's state of the response. 
 *
 * @note 			This has nothing to do with HTTP status codes.
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
native GripBodyHandle:grip_body_from_string(str[]);

/**
 * Destroys body handle
 *
 * @param body		Body to be destroyed
 *  
 * @noreturn
 */
native grip_destroy_body(GripBodyHandle:body);

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
 * @param options	// TODO: Options?
 * @param userData 	User data (can be datapack or anything)
 *  
 * @return			Cancellation handle.
 */
native GripRequestCancellation:grip_request(const uri[], GripBodyHandle:body, GripRequestType:type, const handler[], GripRequestOptionsHandle:options = Invalid_GripRequestOptionsHandle, const userData = 0);

/**
 * Cancel sending of the request and receiving of response.  
 *
 * @param cancellation	Cancellation handle of the request.
 *  
 * @return				Request handle
 */
native grip_cancel_request(GripRequestCancellation:cancellation);

/**
 * Get current error description. Implementation defined.  
 *
 * @note There are certain AMXX limitations to maximum string sizes.  
 *
 * @param buffer	    Output buffer to which description should be written
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
 * @param buffer	    Output buffer to which body should be written
 * @param buffer_size	Maximum length of the buffer.
 *  
 * @noreturn
 */
native grip_get_response_body_string(buffer[], buffer_size);