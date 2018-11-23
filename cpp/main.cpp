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


#include "main.h"
#include "ffi.h"

#include <unistd.h>
#include <iostream>

cell dummy;

void log_error(const void* amx, const char* string) {
	MF_LogError((AMX*)amx, AMX_ERR_NATIVE, "%s", string);
}

void request_handler(cell forward_handle, cell response_handle, const cell *user_data, cell user_data_size) {
	user_data_size = std::max(user_data_size, 0);

	MF_ExecuteForward(
			forward_handle,
			response_handle,
			MF_PrepareCellArray(const_cast<cell*>(user_data), static_cast<ucell>(user_data_size)),
			user_data_size
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

// native GripRequest:grip_request(const uri[], GripBodyHandle:body, GripRequestType:type, const handler[], GripRequestOptionsHandle:options = Invalid_GripRequestOptionsHandle, const userData[] = "", const userDataSize);
// public RequestHandler(GripResponseHandle:handle, const userData[], const userDataSize);
cell AMX_NATIVE_CALL grip_request_amxx(AMX *amx, cell *params) {
	enum { arg_count, arg_uri, arg_type, arg_body_handle, arg_handler, arg_options, arg_user_data, arg_user_data_size };


	const char* uri = MF_GetAmxString(amx, params[arg_uri], 2, &dummy);
	const char* handler_name = MF_GetAmxString(amx, params[arg_handler], 1, &dummy);
	cell handler_forward = MF_RegisterSPForwardByName(amx, handler_name, FP_CELL, FP_ARRAY, FP_CELL, FP_DONE);
	if (handler_forward < 1)
	{
		MF_LogError(amx, AMX_ERR_NATIVE, "Function not found: %s", handler_name);
		return 0;
	}

	cell *user_data = MF_GetAmxAddr(amx, params[arg_user_data]);

	cell options = params[arg_options]; // TODO: Handle options.

	return grip_request(amx, handler_forward, uri, params[arg_type], params[arg_body_handle], request_handler, user_data, params[arg_user_data_size]);
}

AMX_NATIVE_INFO grip_exports[] = {
		{"grip_request", grip_request_amxx},
        {"grip_destroy_body", grip_destroy_body_amxx},
		{"grip_body_from_string", grip_body_from_string_amxx},
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