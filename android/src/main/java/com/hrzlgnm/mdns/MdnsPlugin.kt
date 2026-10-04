// Copyright 2026 hrzlgnm
// SPDX-License-Identifier: MIT

package com.hrzlgnm.mdns

import android.app.Activity
import android.content.Context
import android.net.wifi.WifiManager
import android.os.Build
import androidx.appcompat.app.AppCompatActivity
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

const val LOCAL_NETWORK_ALIAS = "localNetwork"
const val PERMISSION_ACCESS_LOCAL_NETWORK = "android.permission.ACCESS_LOCAL_NETWORK"

// Android 17 made ACCESS_LOCAL_NETWORK mandatory for apps targeting SDK 37.
// Referenced as literals: the permission constant only exists in API 37+.
const val ANDROID_17_API_LEVEL = 37

const val MULTICAST_LOCK_TAG = "tauri-plugin-mdns:discovery"

@TauriPlugin(
  permissions = [
    Permission(strings = [PERMISSION_ACCESS_LOCAL_NETWORK], alias = LOCAL_NETWORK_ALIAS)
  ]
)
class MdnsPlugin(private val activity: Activity) : Plugin(activity) {
  // Held while mDNS browsing is active so the Wi-Fi stack keeps delivering
  // multicast packets (the mdns-sd engine uses raw sockets, not NsdManager).
  // Non-reference-counted with isHeld guards: the Rust side acquires on
  // every browse start and releases once nothing browses anymore, so both
  // entry points stay idempotent and self-healing across activity restarts.
  private var multicastLock: WifiManager.MulticastLock? = null

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

  @Command
  fun acquireMulticastLock(invoke: Invoke) {
    try {
      synchronized(this) {
        var lock = multicastLock
        if (lock == null) {
          val wifi =
            activity.applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
          lock = wifi.createMulticastLock(MULTICAST_LOCK_TAG).apply {
            setReferenceCounted(false)
          }
          multicastLock = lock
        }
        if (!lock.isHeld) {
          lock.acquire()
        }
      }
      invoke.resolve()
    } catch (e: Exception) {
      invoke.reject("Failed to acquire multicast lock: ${e.message}")
    }
  }

  @Command
  fun releaseMulticastLock(invoke: Invoke) {
    try {
      releaseMulticastLockIfHeld()
      invoke.resolve()
    } catch (e: Exception) {
      invoke.reject("Failed to release multicast lock: ${e.message}")
    }
  }

  override fun onDestroy(activity: AppCompatActivity) {
    super.onDestroy(activity)
    // Never leak the lock across activity teardown; the next browse start
    // re-acquires it.
    try {
      releaseMulticastLockIfHeld()
    } catch (_: Exception) {
      // Best effort on teardown.
    }
  }

  private fun releaseMulticastLockIfHeld() {
    synchronized(this) {
      multicastLock?.let {
        if (it.isHeld) {
          it.release()
        }
      }
    }
  }

  private fun grantedState(): JSObject {
    val result = JSObject()
    result.put(LOCAL_NETWORK_ALIAS, PermissionState.GRANTED.toString())
    return result
  }
}
