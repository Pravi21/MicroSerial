# Plugin Development

MicroSerial exposes a stable C ABI for protocol and decoder plugins (`core/include/MicroSerial/plugins/plugin_abi.h`). Plugins are shared objects that export `ms_plugin_query` returning a descriptor with metadata and function pointers.

Future releases will ship a loader that scans `plugins/` at runtime, validates ABI versions, and wires plugin output into GUI views. Authoring guide coming soon.
