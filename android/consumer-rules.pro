# The plugin class is loaded reflectively by name via
# `register_android_plugin`, so it must survive the host app's release
# minification.
-keep class com.hrzlgnm.mdns.MdnsPlugin {
    *;
}
