// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

package com.hrzlgnm.mdns

import android.app.Activity
import android.os.Build
import app.tauri.PermissionState
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.PermissionState
import app.tauri.plugin.Plugin

const val LOCAL_NETWORK_ALIAS = "localNetwork"
const val PERMISSION_ACCESS_LOCAL_NETWORK = "android.permission.ACCESS_LOCAL_NETWORK"

// Android 17 made ACCESS_LOCAL_NETWORK mandatory for apps targeting SDK 37.
// Referenced as literals: the permission constant only exists in API 37+.
const val ANDROID_17_API_LEVEL = 37

@TauriPlugin(
  permissions = [
    Permission(strings = [PERMISSION_ACCESS_LOCAL_NETWORK], alias = LOCAL_NETWORK_ALIAS)
  ]
)
class MdnsPlugin(activity: Activity) : Plugin(activity) {
  // Overrides are deliberately not re-annotated with @Command: the
  // base-class registrations plus virtual dispatch already route to them.
  // Below Android 17 the permission does not exist and local-network
  // access is implicitly granted: both entry points resolve granted so
  // callers never gate (or prompt) where nothing is enforceable.
  override fun checkPermissions(invoke: Invoke) {
    if (Build.VERSION.SDK_INT < ANDROID_17_API_LEVEL) {
      invoke.resolve(grantedState())
    } else {
      super.checkPermissions(invoke)
    }
  }

  override fun requestPermissions(invoke: Invoke) {
    if (Build.VERSION.SDK_INT < ANDROID_17_API_LEVEL) {
      invoke.resolve(grantedState())
    } else {
      super.requestPermissions(invoke)
    }
  }

  private fun grantedState(): JSObject {
    val result = JSObject()
    result.put(LOCAL_NETWORK_ALIAS, PermissionState.GRANTED.toString())
    return result
  }
}
