/*
Copyright Scoir Inc. All Rights Reserved.

SPDX-License-Identifier: Apache-2.0
*/

package vdr

/*
#cgo LDFLAGS: -lindy_vdr
#include "libindy_vdr.h"
#include <stdlib.h>
extern void submitRequestCb(ErrorCode err, const char *response);

void submitRequest(ErrorCode err, const char *response) {
	submitRequestCb(err, response);
}

extern void refreshCb(ErrorCode err);

void refresh(ErrorCode err) {
    refreshCb(err);
}

extern void statusCb(ErrorCode err, const char *response);

void status(ErrorCode err, const char *response) {
	statusCb(err, response);
}

*/
import "C"
