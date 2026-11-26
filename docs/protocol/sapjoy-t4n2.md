# SAPJoy DLL path: T4N2, N2S, T3XS (sapjoyuniapis_t4n2 v2606151)

Scope: device selection, transport dispatch, and the HID frames for the two SAPJoy calls traced so far (`GetVersion`, `SetGamepadStateReport`). The T3XS transfer bodies, the firmware updater, and the non-SAPJoy exports are out of scope. No device was opened.

Labels: `observed-static` (file + line or address), `derived-static` (computed by hand from static facts), `inferred`. No `observed-hw`.

DLL under study: `resources/dfu/sapjoyuniapis_t4n2_x86_v2606151.dll`. Image base 0x10000000. VA = 0x10000000 + RVA. The other DLL versions (v2604013, v2604231, v2605191) and `mod_t3xs.dll` were not compared.

## 1. Device selection and transport

| Model | Wired PID | Wireless PID | Dispatch (observed-static, from GetVersion and SetGamepadStateReport) |
|---|---|---|---|
| T4N2 (Nova 2 Lite) | 0x100F (4111) | 0x1098 (4248) | HID path (section 3) |
| N2S (NOVA VISION) | 0x1141 (4417) | 0x10BD (4285) | HID path (section 3). Inferred from the default branch, not confirmed per PID |
| T3XS | 0x105A (4186) | 0x10BE (4286) | 0x105A takes path A, 0x10BE takes path B (section 4) |

Vendor ID: **0x3537** (13623 decimal), the literal in the selector call (observed-static, `0x10033af0` and `0x10035f70`).

Selector (observed-static, from `0x10033af0` and `0x10035f70`):
1. Enumerate HID devices with VID 0x3537 and the given PID (`0x105A` in `0x10033af0`, `0x10BE` in `0x10035f70`).
2. Walk the returned list. Each entry has a 16-bit field at +0x18 and a next pointer at +0x20. The selector matches entries whose +0x18 field equals 0xFF7A (the field is most likely the usage page, inferred), and the caller branches on the result.
3. When the selector does not match, the code falls through to the HID path.

`GetVersion` (`0x1003c310`) and `SetGamepadStateReport` (`0x10047f90`) both run this dispatch in the same order: `0x10033af0`, then `0x10035f70`, then the HID path.

## 2. Exports used (VA)

| Export | RVA | VA |
|---|---|---|
| SAPJoyUni_Create | 0x48e70 | 0x10048e70 |
| SAPJoyUni_Release | 0x49b70 | 0x10049b70 |
| SAPJoyUniDevice_GetDeviceInfo | 0x3ba10 | 0x1003ba10 |
| SAPJoyUniDevice_GetDevicePID | 0x3bc80 | 0x1003bc80 |
| SAPJoyUniDevice_GetHardwareID | 0x3bea0 | 0x1003bea0 |
| SAPJoyUniDevice_GetMacAddress | 0x3c0b0 | 0x1003c0b0 |
| SAPJoyUniDevice_GetVersion | 0x3c310 | 0x1003c310 |
| SAPJoyUniDevice_SetGamepadStateReport | 0x47f90 | 0x10047f90 |
| SAPJoyUniDevice_SetKeyMappingHSV | 0x3d420 | 0x1003d420 |
| SAPJoyUniDevice_SetStickMapping | 0x43ea0 | 0x10043ea0 |
| SAPJoyUniDevice_UpdateFirmware | 0x45120 | 0x10045120 (firmware, out of scope) |

Full list: 78 exports, from the library's export table (section 11). The non-SAPJoy exports (`GyroParam*`, `KeyRapidFire*`, `N2S_Execute`, `ScreenBrightness*`, `ScreenLanguage*`) are not documented here.

## 3. HID path (shared by T4N2 and N2S)

### 3.1 Exchange helper `0x1004a820` (observed-static)

Arguments: device handle, a 64-byte buffer, length (always 0x40 in these calls), a retry count (3 or 5), and a timeout (0x20 or 5).

1. Drain pending input packets with `0x100551c0`. The helper logs how many it dropped.
2. Write the 64-byte buffer with `0x10057330`. On a write error (-1), log `[WRITE ERROR]`, call `0x10036250`, and retry. The loop runs at most 11 times.
3. After a successful write, read the reply with `0x1004ac30` using the timeout argument.
4. If the loop ends without success, log `[EXECUTE FAIL]`.

The helper's exact return value on success is not recorded here (unresolved).

Vendor-side open failure: the entry points log "unable to open device" and return **0xFFFFFFFA** (-6).

### 3.2 `GetVersion` request (observed-static, `0x1003c310`)

64-byte request, all other bytes zero:

| Byte | Value | Note |
|---|---|---|
| 0 | 0x00 | report id |
| 1 | 0x01 | |
| 2 | 0x01 | |

Sent with retry 3 and timeout 0x20. The reply is read into the same 64-byte buffer. Field copy-out (observed-static, the struct is `SAPJOY_VERSION`, 13 bytes):

| Struct byte | Struct field | Reply byte |
|---|---|---|
| 3 | protocol[0] | 16 |
| 4 | protocol[1] | 17 |
| 6 | firmware[0] | 10 |
| 7 | firmware[1] | 11 |
| 10 | hardware[0] | 6 |
| 11 | hardware[1] | 7 |

Not written by this function: struct bytes 0-2 (engineer), 5 (protocol[2]), 8-9 (firmware[2..3]), and 12 (hardware[2]). Whether they are filled elsewhere is unresolved. The response-read function `0x1004ac30` was not read, so the reply offsets are medium confidence.

The `uchar` argument of the export is not used on the HID path (the function reads only the struct pointer and the device handle).

### 3.3 `SetGamepadStateReport` frame (observed-static, `0x10047f90`, the HID branch)

64-byte frame built from the `uchar` mode argument `m`:

| Byte | Value | Note |
|---|---|---|
| 0 | 0x00 | report id |
| 1 | 0xF3 | |
| 2 | 0x01 | |
| 3 | 0x05 | |
| 4 | m | mode (section 5) |
| 5 | (0xF3 + 0x01 + 0x05 + m) & 0xFF | checksum = low byte of the sum of bytes 1-4 (derived-static) |
| 6-63 | 0x00 | |

Sent with retry 5 and timeout 5. Byte 1 (0xF3) is not a header command id (section 6, ids 1-48), so the frame is not a direct `GS_CMD` (inferred).

Return value on success is not recorded here (unresolved). On open failure: -6.

## 4. T3XS paths (observed-static, GetVersion only)

- PID 0x105A: `GetVersion` calls `0x10032990`. That helper calls `0x100336f0` with arguments (3, 0, 0, 0x40) and copies 13 bytes out (8 bytes, then 4, then 1) into the caller's struct.
- PID 0x10BE: `GetVersion` calls `0x10035390`, the same shape, which calls `0x10035ca0` with arguments (3, 0, 0, 0x40).
- The source offsets inside the transfer buffer are unresolved (the stack layout is not decoded).
- `SetGamepadStateReport` on these PIDs calls `0x10032d20` (0x105A) and `0x10035720` (0x10BE). Not read.

## 5. Mode values (vendor app, observed-static)

`setGamepadStateReport(handle, t)` passes `GAMEPADSTATE_MODE[t]` to the DLL:

| Name | Value |
|---|---|
| Off | 0 |
| Original | 1 |
| Mapped | 2 |

The app defines this table twice and calls `SetGamepadStateReport` from two places. Which mode each device model passes is not verified (the app's arguments were not read).

## 6. Command ids in the header (vendor SDK header, observed-static)

The header defines a `GS_CMD` enum. Id 0 is `GS_CMD_NONE` (placeholder). The HID frames in sections 3.2-3.3 do not use these ids directly. The mapping from header command to HID frame is unresolved.

| Id | Name | Id | Name |
|---|---|---|---|
| 1 | GetDeviceInfo | 25 | SetVibrateTest |
| 2 | GetMacAddress | 26 | SetVibrateLevel |
| 3 | GetVersion | 27 | GetVibrateLevel |
| 4 | SetCustomCalibration | 28 | SetGamepadStateReport |
| 5 | GetCustomCalibration | 29 | SetMotionSenseCurveHSV |
| 6 | SetDPadParam | 30 | GetMotionSenseCurveHSV |
| 7 | GetDPadParam | 31 | SetMotionSenseMapping |
| 8 | SetABXYRotationTest | 32 | GetMotionSenseMapping |
| 9 | SetKeyMappingHSV | 33 | SetConfigPage |
| 10 | GetKeyMappingHSV | 34 | GetConfigPage |
| 11 | SetLightColorTest | 35 | SaveConfigPage |
| 12 | SetLightParam | 36 | ResetConfigPage |
| 13 | GetLightParam | 37 | SetDeviceType |
| 14 | SetLightColorHSV | 38 | GetDeviceType |
| 15 | GetLightColorHSV | 39 | SetStickTriggerCalibration |
| 16 | RecordMacro | 40 | GetStickTriggerCalibration |
| 17 | SetMacroStep | 41 | SetGyroscopeCalibration |
| 18 | GetMacroStep | 42 | GetGyroscopeCalibration |
| 19 | SetStickCurveHSV | 43 | SetISPMode |
| 20 | GetStickCurveHSV | 44 | SetPairMode |
| 21 | SetStickMapping | 45 | SetGamepadParam |
| 22 | GetStickMapping | 46 | GetGamepadParam |
| 23 | SetTriggerParam | 47 | SetReportRate |
| 24 | GetTriggerParam | 48 | GetReportRate |

Result codes (`gs3_ret_t`, from the header): 0 SUCCESS, 1 NOT_SUPPORT, 2 PARAM_ERROR, 3 INVALID_PARAM, 0xFF EMAX.

## 7. Input report layout (`SAPJOYUNI_GAMEPADSTATE`, vendor SDK header, observed-static)

The struct is 38 bytes as declared (bytes 0-37). Bit order follows declaration order, LSB first (inferred from the usual compiler allocation, not checked against the wire). Not verified on the T4N2 wire (no `observed-hw`).

| Byte | Content |
|---|---|
| 0 | analog LS x |
| 1 | analog LS y |
| 2 | analog RS x |
| 3 | analog RS y |
| 4 | analog LT |
| 5 | analog RT |
| 6 | bit0 b, bit1 a, bit2 y, bit3 x, bit4 l1, bit5 l2, bit6 l3, bit7 r1 |
| 7 | bit0 r2, bit1 r3, bit2 minus, bit3 plus, bit4 left, bit5 right, bit6 up, bit7 down |
| 8 | bit0 homw, bit1 vib, bit2 mode, bits3-7 reserved1 |
| 9 | bit0-7 m1..m8 |
| 10 | bit0 m9, bits1-5 reserved2, bit6 s_xb, bit7 s_ns |
| 11-16 | accelerate X, Y, Z: high, low pairs (6 bytes) |
| 17-22 | gyro X, Y, Z: high, low pairs (6 bytes) |
| 23-37 | rgb[15] |

## 8. Unresolved

- `0x1004a520` (open): which HID interface or usage it opens. Not read.
- `0x10057330` (raw write), `0x100551c0` (drain), `0x1004ac30` (reply read), `0x10036250` (error path): whether the transport prepends a report id. Not read.
- `0x100336f0`, `0x10035ca0`, `0x10032d20`, `0x10035720`: T3XS transfer bodies. Not read.
- Meaning of frame bytes 1-3 in both requests. They do not match the header command ids (section 6). `GetVersion` sends 0x01 at byte 1, and `SetGamepadStateReport` sends 0xF3.
- Success return values of `0x1004a820`, `GetVersion`, and `SetGamepadStateReport`.
- Struct bytes not written by `GetVersion` (section 3.2).
- The predicate the app's device-change callback uses, and the set of methods it checks.
- Wireless PIDs 4248, 4285, 4286 are in neither of the app's init watch lists.
- N2S per-PID dispatch: inferred HID, not confirmed.
- Which mode the T4N2 proxy passes to `SetGamepadStateReport`.

## 9. Corrections to other-models.md

- `other-models.md` uses decimal 13623 for the VID, which equals 0x3537. It is consistent with this doc. No edit needed.
- Both of the app's T4N2-style bind paths log "proxyT4N2.bindDevice". Copy-paste, as noted in `other-models.md`. Unchanged here.

## 10. Golden vectors

No `sapjoy-t4n2-vectors.json` was produced. The frame builders run inside the native DLL and need an open HID handle to be exercised. The C2 rule is that golden vectors are outputs of running vendor code, and no HID open or write is allowed here. The frames in sections 3.2-3.3 are documented as derived-static instead. A vector set would need either a device or a mock of the write call `0x10057330` in the DLL.

## 11. Regeneration

The export list comes from the library's PE export table. `0x1003c310` (GetVersion), `0x10047f90` (SetGamepadStateReport) and `0x1004a820` (exchange) were read with a native disassembler. The tooling and its output stay private.
