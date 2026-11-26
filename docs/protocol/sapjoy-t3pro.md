# SAPJoy T3Pro (sapjoyuniapis_x86_v1.0.16.dll) — transport facts

Status: request frames captured by running the vendor library (below); transport verified statically. Labels: `observed-static` (observed in the vendor library's code or import table, with address), `derived-static` (follows from observed code), `inferred` (not confirmed).

## Request frames (vendor-emulated)
A private harness (not published) calls the library's own `ISAPJoyHID` methods (vtable 0x1005d818) on a fake device object under unicorn; the real transport helper runs and its overlapped `WriteFile` is recorded. Output: `sapjoy-t3pro-vectors.json` (deterministic).
- Output report: `07 L G I payload..`, zero-padded to 64 bytes. `L` = payload + 3, `G`/`I` = command group and item. `observed-emulated`.
- Group/item: macro step 01 01, stick curve 02 01, stick mapping 02 03, trigger 03 01, motion curve 04 01, motion mapping 04 03, key mapping 05 01, vibration 06 01, light parameters 07 01, light colour 07 03, D-pad 09 01.
- Payload is the argument struct verbatim (struct layouts in `sapjoy-gs3.md`) for stick mapping, trigger, motion mapping, vibration and light parameters; OpenSir builds these (opensir-core `src/t3pro.rs`, tested against the vectors). Macro step, curves, key mapping, light colour and D-pad rewrite parts of the struct (colour fields and the macro code union are re-encoded); those stay vectors-only.
- Read commands could not be captured: their emulation stops before the write.

## Export dispatch
- `SAPJoyUniDevice_GetVersion` @ 0x10003ac0 (observed-static): returns 0xFFFFFFFF when the handle is null or the length is 0. Dynamic-casts the handle to `ISAPJoyHID` (cast helper at 0x1004d7d8). Returns 0xFFFFFFF6 when the cast fails. Otherwise calls virtual slot +0x9C on the cast object with `(this, length)`. The export holds no HID code of its own.
- `SAPJoyUni_Create` @ 0x10002a60 (observed-static): returns 0xFFFFFFFF on a null argument, a null `param_1[1]`, or `param_2 == 0`. Builds a log path under the known-folder path with `sapjoyapis.dev.log`. Allocates 0x60 bytes, zeroes them, runs the constructor `0x10011280`. Returns 0xFFFFFFFC if the constructor returns null, else stores the object and returns 0.
- `SAPJoyUni_Release` @ 0x10002c20 (observed-static): calls vtable slot 0 with arg 1 (deleting destructor) if non-null, then logs "SAPJoyTracer is destroyed".

## Device object constructor
- `0x10011280` (observed-static): vtable `0x1005DCF8` at +0. Copies a 0x58-byte config block to `this+4..+0x5C`. Two 16-entry UTF-16 string-list loops. Creates a 0x58-byte sub-object (vtables `0x1005E4B4` at +0, `0x1005E498` at +4). Sub-object +0x40 = `CreateEventW`, +0x48 = GUID from `HidD_GetHidGuid`.
- Device fields used later (observed-static, from `0x10011e90` and `0x10012320`): +4 HID handle, +8 hotplug notification handle, +0xC read buffer object, +0x94 worker thread handle, +0x98 worker thread id.

## Write path: transact helper `0x10012060` (observed-static)
- Signature `(this, cmd_obj, cmd_ref, want_response)`. Takes the lock at `this+0x24` (`0x100166f8`).
- Asks the command object (virtual +8) to fill a buffer delimited by `[begin, end]`. The write length is `end − begin`.
- Loop of up to 3 attempts: `CreateEventW`, `WriteFile` on handle `this+4` (overlapped), `WaitForSingleObject` with the timeout from command virtual +12. A wait failure is logged.
- If `want_response` is set and the wait did not time out, calls `0x10012840` for the response. Maps the boolean to 0 (true) or 0xFFFFFFFD (false). Three timeouts give 0xFFFFFFF5.
- Cleanup releases the lock and drops the command reference.
- `WriteFile` (IAT 0x1005108C) has 7 call sites (observed-static). Only 0x100121C4, inside `0x10012060`, sends HID frames. The other six are CRT `_write` log writers (fd table, console code page). They are not part of the protocol.

## Read path (observed-static)
- `0x10012320`: starts an overlapped read. Calls `ReadFileEx(handle=this+4, buf=*(this+0xC), len = buf[1] − buf[0], overlapped = buf+3, completion = 0x10012570)`. On failure logs "Failed in calling ReadFileEx: %d".
- `0x10012570`: completion routine. Only calls `0x10012370(error, this, count)`.
- `0x10012370`: completion handler. The read pump re-arms itself after every completion by calling `0x10012320` again.
  - Errors (non-zero first arg) are logged "Failed in lpCompletionRoutine: %d" and return.
  - Accepts the frame only if: the received length equals the read buffer size, `byte[0] == (object+0x9E) − 1`, and `byte[1] != 0`. Otherwise the frame is dropped and the read re-armed.
  - The state at object +0x50 is a sentinel. It is 0x7FFFFFFF when idle. A completion that finds 0x7FFFFFFF writes 0x7FFFFFFE and aborts through a fatal path.
  - Then the handler takes the lock at +0x24, runs an optional filter callback, and signals the waiter with `0x10016709` / `0x1001770e`.
- Object +0x9E is read by the completion check (`byte[0]` must equal it minus 1). Its writer is not located yet. `0x10004390` returns `(object+0x9E) == 5`. Whether the same field is involved is `inferred`.

## Stop path: `0x10011e90` (observed-static)
- Posts `WM_QUIT` (0x12) to the worker thread id at +0x98, then waits on the worker handle at +0x94 with INFINITE timeout. Self-join is a fatal path.
- `CancelIoEx(handle, overlapped)` cancels the pending read.
- Hotplug: if +8 is set, `UnregisterDeviceNotification` is called (User32 through `GetProcAddress`), then +8 is cleared.
- `CloseHandle(handle)` unless it is 0 or `INVALID_HANDLE_VALUE`, then clears +4. Frees the read buffer object.

## Serializer: SetMacroStep payload `0x100020e0` (observed-static)
- Writes into a byte vector through `this+8` (begin), `this+0xC` (end), `this+0x10` (capacity). Writes come one byte at a time.
- Bytes 0–7: copied from the argument, `arg[0]` … `arg[7]`. Byte 6 selects the variant below.
- Byte 6 switch:
  - 0: writes 4 zeros (bytes 8–11), then jumps to the tail.
  - 1: writes `arg[8..11]` (bytes 8–11), then jumps to the tail.
  - 2: writes `arg[8]`, `arg[9]` (bytes 8–9), then falls through to the middle.
  - 3: writes 2 zeros (bytes 8–9), then falls through to the middle.
  - Middle (cases 2 and 3): writes `arg[10]`, `arg[11]`.
- Tail: writes `arg[12]` … `arg[15]`.
- Total frame length for SetMacroStep: **16 bytes**, observed-static. For each subtype, bytes 8–11 hold payload or zeros, and bytes 12–15 are always copied from the argument.
- The argument struct layout (the caller's `param_1`) is not decoded. The header bytes 0–7 are not labelled yet.

## SetMacroStep builder `0x10005830` (observed-static)
- Allocates a 0x30-byte ref-counted object. Vtable `std::_Ref_count_obj2<CSAPJoySetMacroStepCmd>` at +0. Command object at +0x0C.
- Ref +0x10 (command +0x04) halfword = 0x0101. Its meaning is not confirmed. The serializer does not read it.
- Ref +0x2C (command +0x20) = 500. Read as a timeout in ms (`inferred`).
- Calls the serializer on command `+0x0C`, then `0x10012060(this-0xE8, cmd, ref, 1)`.
- **Caveat (observed-static):** the return value of `0x10012060` is ignored. OpenSir must check write and wait status itself.

## Response check `0x10001de0` (observed-static)
- Reads the response through command virtual +0x18. Requires length > 4. If `byte[2]` and `byte[3]` are both zero, logs "Command{ 0x%02X, 0x%02X } not supported" and returns 0xFFFFFFFF.
- If `byte[4] != 0`, logs "execute failed: 0x%02X", stores `byte[4]` in the output pointer if given, and returns 1. Otherwise 0.
- Meaning of bytes 2–4 is `inferred` from the log text. The completion handler's `byte[0]` and `byte[1]` checks are a separate, stronger filter and are observed.

## Imports (observed-static, from the library's import table)
- HID.DLL: `HidD_GetHidGuid`, `HidD_GetPreparsedData`, `HidD_FreePreparsedData`, `HidD_GetAttributes`, `HidP_GetCaps`, `HidP_GetValueCaps`. No `HidD_SetOutputReport`, `HidD_SetFeature`, or `HidD_GetInputReport`.
- SETUPAPI.dll: `SetupDiGetClassDevsW`, `SetupDiEnumDeviceInterfaces`, `SetupDiGetDeviceInterfaceDetailW`, `SetupDiGetDevicePropertyW`, `SetupDiDestroyDeviceInfoList`.
- KERNEL32.dll (I/O): `CreateFileW`, `ReadFile`, `ReadFileEx`, `WriteFile`, `CancelIoEx`, `CreateEventW`, `WaitForSingleObject`, `SetEvent`, `CloseHandle`.
- Conclusion (derived-static): transport is overlapped `WriteFile` and `ReadFileEx` on a HID handle opened with `CreateFileW`. Feature and input report calls are not used.

## Command ids (vendor SDK header, observed-static)
- `GS_CMD_GetVersion` = id 3 (request none, response SAPJOY_VERSION).
- `GS_CMD_RecordMacro` = id 16 (request RecordMacroParam).
- `GS_CMD_SetMacroStep` = id 17 (request SAPJOYUNI_MACROSTEP).
- `GS_CMD_GetMacroStep` = id 18 (request MacroStep, response SAPJOYUNI_MACROSTEP).
- Mapping from id to frame byte 0 is not confirmed (`inferred`). Do not assume it.

## Cross-check with T4N2 (read-only, `docs/protocol/sapjoy-t4n2.md`)
- Both DLLs use overlapped `WriteFile` (T4N2 helper `0x10057330`, T3Pro `0x100121C4`). Both read replies, but T3Pro uses a continuous `ReadFileEx` pump while T4N2 reads synchronously (`0x1004ac30`).
- T4N2 frames are 64 bytes with report id 0x00 at byte 0. The T3Pro SetMacroStep serializer produces 16 bytes (write length `end − begin`). Whether T3Pro wraps it to a 64-byte report before `WriteFile` is not confirmed (`inferred`: no; the write length in `0x10012060` is taken straight from the vector).
- T4N2 reply check is not the same as the T3Pro completion check. Do not reuse one for the other.

## Open questions
1. Which caller fills the argument of `0x100020e0` and what bytes 0–7 are.
2. Writer of object +0x9E, and whether it is the same as the `0x10004390` state check.
3. Read buffer size. The completion handler checks it against the buffer vector.
4. Meaning of the 0x0101 halfword in the SetMacroStep command object.
5. Vtable targets for `0x1005DCF8`, `0x1005E4B4`, `0x1005E498`, and slot +0x9C on `ISAPJoyHID`.
6. Command-to-frame map for the remaining GS_CMD ids, and the six CRT writers (excluded from protocol).
