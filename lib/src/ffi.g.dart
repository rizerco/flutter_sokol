// ignore_for_file: type=lint, unused_import
import 'dart:ffi' as ffi;

@ffi.Native<ffi.Void Function(ffi.Uint64)>()
external void randomize_clear_color(int address);

@ffi.Native<ffi.Uint64 Function()>()
external int state_pointer();
