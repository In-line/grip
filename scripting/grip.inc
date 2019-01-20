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
	GripResponseStateTimeout = 4,
}

enum GripHTTPStatus {
    GripHTTPStatusContinue = 100,
    GripHTTPStatusSwitchingProtocols = 101,
    GripHTTPStatusProcessing = 102,
    GripHTTPStatusOk = 200,
    GripHTTPStatusCreated = 201,
    GripHTTPStatusAccepted = 202,
    GripHTTPStatusNonAuthoritativeInformation = 203,
    GripHTTPStatusNoContent = 204,
    GripHTTPStatusResetContent = 205,
    GripHTTPStatusPartialContent = 206,
    GripHTTPStatusMultiStatus = 207,
    GripHTTPStatusAlreadyReported = 208,
    GripHTTPStatusImUsed = 226,
    GripHTTPStatusMultipleChoices = 300,
    GripHTTPStatusMovedPermanently = 301,
    GripHTTPStatusFound = 302,
    GripHTTPStatusSeeOther = 303,
    GripHTTPStatusNotModified = 304,
    GripHTTPStatusUseProxy = 305,
    GripHTTPStatusTemporaryRedirect = 307,
    GripHTTPStatusPermanentRedirect = 308,
    GripHTTPStatusBadRequest = 400,
    GripHTTPStatusUnauthorized = 401,
    GripHTTPStatusPaymentRequired = 402,
    GripHTTPStatusForbidden = 403,
    GripHTTPStatusNotFound = 404,
    GripHTTPStatusMethodNotAllowed = 405,
    GripHTTPStatusNotAcceptable = 406,
    GripHTTPStatusProxyAuthenticationRequired = 407,
    GripHTTPStatusRequestTimeout = 408,
    GripHTTPStatusConflict = 409,
    GripHTTPStatusGone = 410,
    GripHTTPStatusLengthRequired = 411,
    GripHTTPStatusPreconditionFailed = 412,
    GripHTTPStatusPayloadTooLarge = 413,
    GripHTTPStatusUriTooLong = 414,
    GripHTTPStatusUnsupportedMediaType = 415,
    GripHTTPStatusRangeNotSatisfiable = 416,
    GripHTTPStatusExpectationFailed = 417,
    GripHTTPStatusImATeapot = 418,
    GripHTTPStatusMisdirectedRequest = 421,
    GripHTTPStatusUnprocessableEntity = 422,
    GripHTTPStatusLocked = 423,
    GripHTTPStatusFailedDependency = 424,
    GripHTTPStatusUpgradeRequired = 426,
    GripHTTPStatusPreconditionRequired = 428,
    GripHTTPStatusTooManyRequests = 429,
    GripHTTPStatusRequestHeaderFieldsTooLarge = 431,
    GripHTTPStatusUnavailableForLegalReasons = 451,
    GripHTTPStatusInternalServerError = 500,
    GripHTTPStatusNotImplemented = 501,
    GripHTTPStatusBadGateway = 502,
    GripHTTPStatusServiceUnavailable = 503,
    GripHTTPStatusGatewayTimeout = 504,
    GripHTTPStatusHttpVersionNotSupported = 505,
    GripHTTPStatusVariantAlsoNegotiates = 506,
    GripHTTPStatusInsufficientStorage = 507,
    GripHTTPStatusLoopDetected = 508,
    GripHTTPStatusNotExtended = 510,
    GripHTTPStatusNetworkAuthenticationRequired = 511,
};

/**
 * Gets state of the response.
 *
 * @note    		This has nothing to do with HTTP status codes.
 *
 * @return			Returns current response state.
 */
native GripResponseState:grip_get_response_state();


/**
 * Gets HTTP status code of the response.
 *
 * @return			Returns current response state.
 */
native any:grip_get_response_status_code();

/**
 * Returns whether request exists/active. 
 *
 * @note 			This has nothing to do with HTTP status codes.
 *
 * @param request	Request handle.
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
native GripRequestOptions:grip_create_default_options(Float:timeout = -1.0);

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
 * @param headers	Array of header pairs.
 * @param headers_count	Count of the header pairs.
 * @param timeout	timeout -1 to disable, >0 to enable timeout.
 *
 * @return 		Request options handle.
 */
stock grip_create_create_options(const headers[][2][], const headers_count, Float:timeout = -1.0) {
	new GripRequestOptions:options = grip_create_default_options(timeout);
	
	for(int i = 0; i < headers_count; ++i) {
		grip_options_add_header(options, headers[i][0], headers[i][1]);
	}
	
	return options;
}

