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

#include "amxxmodule.h"
#include "main.h"
#include "ffi.h"

#include <unistd.h>

cell dummy;

void log_error(const void* amx, const char* string) {
	MF_LogError((AMX*)amx, AMX_ERR_NATIVE, "%s", string);
}

void request_handler(cell forward_handle, cell response_handle, cell user_data) {
	MF_ExecuteForward(
			forward_handle,
			response_handle,
			user_data
	);
	MF_UnregisterSPForward(forward_handle);
}

//native GripBodyHandle:grip_body_from_string(str[]);
cell AMX_NATIVE_CALL grip_body_from_string_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_str };
	const char* str = MF_GetAmxString(amx, params[arg_str], 3, &dummy);
	return grip_body_from_string(amx, str);
}

//native grip_destroy_body(GripBodyHandle:body);
cell AMX_NATIVE_CALL grip_destroy_body_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_body };

	return grip_destroy_body(amx, params[arg_body]);
}

// native GripRequest:grip_request(const uri[], GripBodyHandle:body, GripRequestType:type, const handler[], GripRequestOptionsHandle:options = Invalid_GripRequestOptionsHandle, const userData);
// public RequestHandler(GripResponseHandle:handle, const userData);
cell AMX_NATIVE_CALL grip_request_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_uri, arg_body_handle, arg_type, arg_handler, arg_options, arg_user_data };


	const char* uri = MF_GetAmxString(amx, params[arg_uri], 2, &dummy);
	const char* handler_name = MF_GetAmxString(amx, params[arg_handler], 1, &dummy);
	cell handler_forward = MF_RegisterSPForwardByName(amx, handler_name, FP_CELL, FP_DONE);
	if (handler_forward < 1)
	{
		MF_LogError(amx, AMX_ERR_NATIVE, "Function not found: %s", handler_name);
		return 0;
	}

	return grip_request(amx, handler_forward, uri, params[arg_body_handle], params[arg_type], request_handler, params[arg_options], params[arg_user_data]);
}

cell AMX_NATIVE_CALL grip_cancel_request_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_cancellation };
	return grip_cancel_request(amx, params[arg_cancellation]);
}

cell AMX_NATIVE_CALL grip_get_response_state_amxx(AMX *amx, cell*) {
    return grip_get_response_state(amx);
}

cell AMX_NATIVE_CALL grip_is_request_active_amxx(AMX *, cell *params) {
	enum { arg_count, arg_request_id };
	return grip_is_request_active(params[arg_request_id]);
}

cell AMX_NATIVE_CALL grip_get_error_description_amxx(AMX *amx, cell *params) {
  enum { arg_count, arg_buffer, arg_buffer_size};

  char buffer[params[arg_buffer_size]];
  cell ret = grip_get_error_description(amx, &buffer[0], params[arg_buffer_size]);

  MF_SetAmxString(amx, params[arg_buffer], &buffer[0], params[arg_buffer_size]);

  return ret;
}

cell AMX_NATIVE_CALL grip_get_response_body_string_amxx(AMX *amx, cell *params) {
  enum { arg_count, arg_buffer, arg_buffer_size};

  char buffer[params[arg_buffer_size]];

  cell ret = grip_get_response_body_string(amx, &buffer[0], params[arg_buffer_size]);

  MF_SetAmxString(amx, params[arg_buffer], &buffer[0], params[arg_buffer_size]);

  return ret;
}

cell AMX_NATIVE_CALL grip_parse_response_body_as_json_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_buffer, arg_buffer_size};

	char buffer[params[arg_buffer_size]];

	cell ret = grip_parse_response_body_as_json(amx, &buffer[0], params[arg_buffer_size]);

	MF_SetAmxString(amx, params[arg_buffer], &buffer[0], params[arg_buffer_size]);

	return ret;
}

cell AMX_NATIVE_CALL grip_destroy_json_value_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_json_value};
	return grip_destroy_json_value(amx, params[arg_json_value]);
}

cell AMX_NATIVE_CALL grip_create_default_options_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_timeout};

	return grip_create_default_options(amx, amx_ctof(params[arg_timeout]));
}

cell AMX_NATIVE_CALL grip_destroy_options_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_options_handle};

	return grip_destroy_options(amx, params[arg_options_handle]);
}
//     cell grip_options_add_header(const void* amx, cell options_handle, const char* header_name, const char* header_value);
cell AMX_NATIVE_CALL grip_options_add_header_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_options_handle, arg_header_name, arg_header_value};

	return grip_options_add_header(amx, params[arg_options_handle],
			MF_GetAmxString(amx, params[arg_header_name], 0, &dummy),
			MF_GetAmxString(amx, params[arg_header_name], 1, &dummy));
}

AMX_NATIVE_INFO grip_exports[] = {
	{"grip_request", grip_request_amxx},
    {"grip_destroy_body", grip_destroy_body_amxx},
    {"grip_body_from_string", grip_body_from_string_amxx},
    {"grip_cancel_request", grip_cancel_request_amxx},
    {"grip_get_response_state", grip_get_response_state_amxx},
    {"grip_is_request_active", grip_is_request_active_amxx},
    {"grip_get_error_description", grip_get_error_description_amxx},
    {"grip_get_response_body_string", grip_get_response_body_string_amxx},
    {"grip_parse_response_body_as_json", grip_parse_response_body_as_json_amxx},
    {"grip_destroy_json_value", grip_destroy_json_value_amxx},
	{"grip_create_default_options", grip_create_default_options_amxx},
	{"grip_destroy_options", grip_destroy_options_amxx},
	{"grip_options_add_header", grip_options_add_header_amxx},
    {nullptr, nullptr}
};

void OnAmxxAttach()
{
	char configFilePath[MAX_PATH];
	MF_BuildPathnameR(configFilePath, sizeof(configFilePath), "%s/grip.ini", MF_GetLocalInfo("amxx_configsdir", "addons/amxmodx/configs"));

	grip_init(log_error, configFilePath);
	MF_AddNatives(grip_exports);
}

void OnAmxxDetach()
{
	grip_deinit();
}

void StartFrame() {
	grip_process_request();
	SET_META_RESULT(MRES_IGNORED);
}
