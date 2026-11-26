# SAPJoy T3XS and N2S wire notes

Companion to `sapjoy-t4n2.md`. That file keeps the T4N2 selector, the 64-byte exchange, and the GetVersion frame. This file covers:

1. T3XS transport (PIDs 0x105A wired, 0x10BE wireless) in the T4N2 lib, and `mod_t3xs.dll`.
2. N2S-only exports in the T4N2 lib: frame bytes, payloads, and how the app reaches them.
3. Whether the wireless PIDs 0x1098, 0x10BD, 0x10BE reach the DLL, and what the app's device-change predicate and watch list contain.

Do not edit `sapjoy-t4n2.md` from this file. Where it is wrong, the correction is here.

## 0. Labels and sources

- `observed-static`: observed in the vendor library code (address cited), in the vendor app's behaviour and configuration, or in an export table entry.
- `derived-static`: a byte, checksum, or length computed by hand from observed code.
- `inferred`: a placement or meaning not fully confirmed (stack-offset mapping, argument slot order, unread helper).

Addresses are the DLL image base 0x10000000.

## 1. Devices and enumeration

| Item | Value | Label |
|---|---|---|
| Vendor VID | 0x3537 (13623) | observed-static (vendor app) |
| HID usage page used by all paths | 0xFF7A (stored as signed short `-0x86` at HID node offset +0x18; next node at +0x20) | observed-static |
| T3XS wired | VID 0x3537, PID 0x105A (4186) | observed-static (app device table) |
| T3XS wireless | VID 0x3537, PID 0x10BE (4286) | observed-static (app device table) |
| N2S wired | PID 0x1141 (4417) | observed-static (`N2S_Execute` 0x10031bb0) |
| N2S wireless | PID 0x10BD (4285) | observed-static (`N2S_Execute` 0x10031bb0) |
| T4N2 wireless ConfigMode | PID 0x1098 (4248) | observed-static (app device table) |
| Second VID probed by DLL | 0x413E (16702) | observed-static (`0x100345d0`) |

Correction to `sapjoy-t4n2.md`: the T3XS selectors `0x10033af0` (path A) and `0x10035f70` (path B) do **not** filter by PID. Both call the enumerator `0x10054160(0x3537)` and return nonzero when any node has usage 0xFF7A. The PID is never passed. `observed-static`.

`N2S_Execute` is the one path that filters by PID: it opens `0x10030860(0x3537, 0x1141, -0x86)` and falls back to `(0x3537, 0x10BD, -0x86)`. `observed-static`.

## 2. T3XS transport (SAPJoy frame family, DLL-internal)

### 2.1 Routing: `T3XS_Execute` (0x10034260)

Signature: `T3XS_Execute(char cmd, uint *payload, uint payload_len, uint *out, size_t out_len)`. `observed-static`.

1. Enumerate VID 0x3537. If any node has usage 0xFF7A, call `0x10031f30(cmd, payload, len, out, out_len)`. That is **path A**. `observed-static`.
2. Otherwise call `0x100345d0()` (enumerate VID 0x413E, usage 0xFF7A). If nonzero: one-time init (guard flag 0x101be657, init `0x100545b0`), enumerate VID 0x413E, open the first 0xFF7A node, build the frame with `0x10034620`, read with `0x10033b40(h, cmd, out, out_len, 2000)`, close `0x100540f0`. No handle: return `0xfffffffa`. This is **path B**. `observed-static`.
3. Otherwise (reached only when step 2 found nothing): call `0x10035f70()`. Zero returns `0xfffffff0`. Nonzero runs the same open/build/read sequence on VID 0x3537, usage 0xFF7A, with guard flag 0x101be659. `observed-static`.

`derived-static`: step 3's predicate (VID 0x3537, usage 0xFF7A) is the same as step 1's. Step 1 would already have returned for that case. Step 3's success path is therefore reachable only if enumeration results differ between the two calls. Not confirmed.

### 2.2 Transfer helpers

| Function | Role | Label |
|---|---|---|
| `0x100545b0` | one-time HID init; logs `_hid_init: OK/FAIL` | observed-static |
| `0x10054160(vid)` | enumerate HID nodes for a VID, returns list | observed-static |
| `0x10054460(list)` | free the list | observed-static |
| `0x10054f40(path)` | open by path (details in `sapjoy-t4n2.md` §3) | observed-static |
| `0x100540f0(h)` | close | observed-static |
| `0x10057330(h, buf, len)` | raw `WriteFile`, no report-id prepend, zero-pads to the output report size | observed-static |
| `0x10033b40(h, cmd, out, out_len, 2000)` | T3XS reader, see §2.4 | observed-static |

Path A and path B transfer bodies are `0x100336f0` and `0x10035ca0`. Both are 6953 bytes and differ only in the builder they call (`0x10033f50` for A, `0x10034620` for B). `observed-static`.

`0x10031f30` (the path A entry point used by `T3XS_Execute` step 1) is the same transport as `0x100336f0`, as a standalone function. Its error return when no device is found is `0xfffffffa` with a UTF-8 log line. `observed-static`.

### 2.3 Frame (T3XS SAPJoy frame)

Builders `0x10033f50` (A) and `0x10034620` (B) behave identically (`observed-static`). The 64-byte buffer is zeroed first.

| Byte | Value | Notes |
|---|---|---|
| 0 | `0x00` | report prefix byte, zeroed |
| 1 | `0xA5` | header |
| 2 | `payload_len + 5` | length |
| 3 | `0xC0` | family tag |
| 4 | `cmd` | command id |
| 5 .. 4+len | payload | copied from the caller |
| 5+len | checksum | `sum(bytes 1 .. 4+len) & 0xFF` (`derived-static`) |

- **Write length** = `payload_len + 6` (not 64). The writer zero-pads to the output-report size. `observed-static` for the length expression.
- Example, GetVersion (cmd 0x03, no payload): `00 A5 05 C0 03 6D`, checksum `(A5+05+C0+03) & 0xFF = 0x6D`. Write length 6. `derived-static`.
- Example, SetGamepadStateReport (cmd 0x1C, 1-byte payload `m`): `00 A5 06 C0 1C m cs`, `cs = (0x87 + m) & 0xFF`. Write length 7. `derived-static`.

### 2.4 Reply (`0x10033b40`)

- Loops `0x100551c0(h, buf, len, 5)`, zeroing the buffer each pass.
- Accepts a reply only when the return is `> 5`, byte 0 = `0xA5`, byte 2 = `0xC1`, and byte 3 equals the requested command. `observed-static`.
- Payload starts at byte 5. `observed-static`. Reply length in byte 1: `inferred`.
- Read timeout 2000 ms. `observed-static`.

### 2.5 Commands

Command byte 4 of the frame is the id that the vendor app passes to `T3XS_Execute` for the app-driven commands. `observed-static`:

| Id | Name (app) | Payload | Reply |
|---|---|---|---|
| 0x31 (49) | `T3XSGyroParamSet` | `SAPJOY_GYROPARAM` | n/a |
| 0x32 (50) | `T3XSGyroParamGet` | 1 byte | n/a |
| 0x33 (51) | `SetStickCurveHSVT3XS` | `SAPJOYUNI_STICKCURVE_5POINTS_HSV` | n/a |
| 0x34 (52) | `GetStickCurveHSVT3XS` | `[t, r]` (2 bytes) | n/a |
| 0x35 (53) | `T3XSReceiverVersionGetAsync` | none | 2 bytes |

Commands used only inside the DLL (not reached from the app):

- **GetVersion, cmd 0x03** (`0x10032990` path A, `0x10035390` → `0x10035ca0` path B). `observed-static`. On success the reply is copied into the version struct. The byte mapping (reply 5..12 → struct 0..7, 13..16 → 8..11, 17 → 12) is `inferred`; it has not been checked against the copy code.
- **SetGamepadStateReport, cmd 0x1C**, 1-byte payload = caller's mode byte (`inferred`). Wrappers `0x10032d20` (A, calls `0x100336f0`) and `0x10035720` (B, calls `0x10035ca0`). `observed-static`.

Whether cmd 0x1C is an app-level id is not established.

### 2.6 Dispatch from `SetGamepadStateReport` (0x10047f90)

Order observed in `0x10047f90` (`observed-static`):

1. Guard (a global state is not 0x7fffffff) → call `0x10033af0()` (path A selector). Nonzero → `0x10032d20` (path A SetGamepadState).
2. Else call `0x10035f70()` (path B selector). Nonzero → `0x10035720`.
3. Else the T4N2 frame branch (`00 F3 01 05 m cs`, §4.3).

So the T4N2 frame is sent only when neither T3XS selector finds a usage-0xFF7A node on VID 0x3537. The T4N2 family uses the same usage page (`sapjoy-t4n2.md` §2). That overlap is an open contradiction, see §6.

## 3. `mod_t3xs.dll` (second DLL family, not the SAPJoy frame)

### 3.1 Exports

81 exports, all named `GameSir_*` (from the DLL's export table, `observed-static`). Groups, with a few addresses:

- Macro and key: `GameSir_ClearMacro` (0xa3a0), `GameSir_GetKeyMapping` (0x3ed0), `GameSir_GetKeyReportType` (0x2550), `GameSir_GetLeftStickDpadSwap` (0x40b0), `ResetKeyMapping`, `ResetAllKeyMappings`, `Start/StopMacroRecording`, `Start/StopKeyDataReporting`, `RegisterKeyDataCallback`, Get/Set Macro.
- Device and profile: `GameSir_GetDeviceInfo` (0x20e0), `GameSir_GetBatteryInfo` (0x1c90), `GameSir_EnterDFUMode` (0x1a90), `GameSir_GetCurrentProfile` (0x1ec0), Get/Set/SwitchProfile, `RestoreFactorySettings`, `GetDockAutoStartStop` (0x2310), `GetPollRate`/`SetPollRate`.
- Gyro: `GameSir_GetGyroAspectRatio` (0x81a0), `GetGyroCurve` (0x8370), `GetGyroDPI` (0x85a0), `GetGyroDeadzone` (0x8760), `GetGyroFunction` (0x8940), `GetGyroMappingConfig` (0x8b20), `GetGyroOutputMode` (0x8d40), `GetGyroStartConfig` (0x8f10), `GetGyroXAxisConfig` (0x90f0), plus Set counterparts.
- Stick, trigger, vibration: Get/Set Stick*, Trigger*, Vibration*, `TriggerRealtimeVibration`, `GetContinuousTrigger` (0x3b10), `GetDpadDirectionLock` (0x3d00), RapidFire Get/Set, Quick trigger.

### 3.2 Device open (`0x1000d3c0`)

- One-time init (guard flag 0x1004c440, init `0x10013020`, logs `hid_init: OK` / `hid_init: FAIL`). `observed-static`.
- Enumerate with `0x10012d50` (argument: a global at 0x1004b034). Walk nodes and match the 16-bit field at node+0x18 against a global at 0x1004b038. Open the match by path with `0x100139b0`. Free the list with `0x10012fd0`. `observed-static`.
- **The filter is usage-only.** There is no PID check in this function. The globals at 0x1004b034 (VID or list) and 0x1004b038 (usage) are not read. `inferred` they are 0x3537 and 0xFF7A.
- No device: log a UTF-8 line (text not decoded), return 0. `observed-static`.

### 3.3 Frame builder (`0x1000cce0`)

Arguments: `(dev, cmd, sub, payload, payload_len, out, out_len)`. `observed-static`.

| Byte | Value | Notes |
|---|---|---|
| 0 | `0x00` | zeroed |
| 1 | `0xA5` | `observed-static` |
| 2 | `payload_len + 6` | `observed-static` |
| 3 | `0xC7` | family tag (differs from SAPJoy `0xC0`) |
| 4 | `cmd` | |
| 5 | `sub | 0x80` | sub-command with high bit set |
| 6 .. | payload | copied from the caller, only when `payload_len != 0` |
| payload_len+6 | checksum | weighted byte sum built in an XMM register chain. **Not decoded.** |

- Write: `0x1000d220(dev, &buf, payload_len + 1)`. The write length of `payload_len + 7` is `inferred` from this call shape.
- Read: `0x1000ceb0(dev, &buf, out_len + 6, 1000, cmd)`. Timeout 1000 ms (`observed-static`). On success with `out_len != 0`, copies `out_len` bytes from offset 6.
- Reply byte validation is not decoded.

Example, `GameSir_GetGyroCurve` (`0x10008370`): cmd 0x05, sub 4 (byte 5 = `0x84`), 1 payload byte, 13-byte reply. Frame `00 A5 07 C7 05 84 <p> <cs>`, write length 8, read length 19. The reply mapping from the first output byte into the output struct is `inferred` beyond the call shape.

### 3.4 Loader and relation to T3XS SAPJoy transport

- No `mod_t3xs` string in the vendor app's JavaScript, `sapjoyuniapis_t4n2_x86_v2606151.dll`, `sapjoyuniapis_x86_v1.0.16.dll`, or `sapjoyuniapis_run.exe`. The loader was **not located** (bounded: not searched in the `.node` files or other DLLs).
- `mod_t3xs.dll` and the SAPJoy T3XS path (§2) both use usage 0xFF7A, but different frame tags (`C7` vs `C0`), different sub-command handling, and different timeouts (1000 vs 2000 ms). Whether `mod_t3xs.dll` is ever used for PID 0x105A or 0x10BE is not established. Do not merge the two frame families.
- Other DLLs present in the dfu folder (`sapjoyuniapis_t4n2_x86_v2604013/2604231/2605191/2606151.dll`, `sapjoyuniapis_x86_v1.0.11/1.0.15/1.0.16.dll`) were not compared for these exports.

## 4. N2S-only exports (T4N2 lib, `sapjoyuniapis_t4n2_x86_v2606151.dll`)

### 4.1 Two transports, one frame grammar

The N2S_Execute path and the export path use the same frame layout for their N2S commands:

`byte0 = 0x00 | byte1 = cmd | byte2 = sub | byte3 = len + 4 | bytes 4 .. len+3 = payload | byte len+4 = checksum`

Checksum = `sum(bytes 1 .. len+3) & 0xFF`. `derived-static`.

- **Transport N (`N2S_Execute`, 0x10031bb0)**: PID-filtered open (0x1141, then 0x10BD), frame builder `0x10031390`, write length `len + 5`, read `0x10030bf0(h, cmd, buf, len, 2000)`. Probe `0x10030580(0x3537, 0x10BD, -0x86)` runs only when no 0xFF7A node was found; its failure returns `0xfffffff0`. Other error codes: `0xfffffffa` open failed; `0xfffffff5` frame builder failed; `0xfffffff4` read failed; 0 success. `observed-static`. The `"Send: "` debug path (helper `0x10031810`) is not read.
- **Transport E (exports, 0x10047f90 family)**: opener `0x1004a520()` (null → `0xfffffffa`, log "unable to open device"), exchange `0x1004a820(h, buf, 0x40, n, n)`. Fixed 64-byte buffer. `observed-static`. The node match inside `0x1004a520` is usage-only is `inferred`: its PID filter has not been read, by analogy with the T3XS selectors.
- Gate: `ScreenBrightnessSet` returns `0xfffffffd` when both T3XS selectors (`0x10033af0`, `0x10035f70`) are nonzero. `observed-static`. Other N2S-family setters use the same gate pattern (`inferred` for all of them; checked on `ScreenBrightnessSet` only).

### 4.2 App command and action enums (N2S_Execute callers)

The vendor app's async N2S helper takes `(cmd, action, payload, payloadLen, bufLen)` and calls the DLL's `N2S_Execute`. `observed-static`.

- `N2S_COMMAND`: Heartbeat 96 (0x60), StickCenter 251 (0xFB), ProfileSwitch 252 (0xFC), FineCalibration 98 (0x62), KeyAdvanceMapping 99 (0x63), MotionAdvanceConfig 100 (0x64), StickAdvanceConfig 101 (0x65), ProfileName 253 (0xFD).
- `N2S_ACTION`: Notify 0, Query 0, Write 1, Reset 2, SelectProfileDataAccess 2.

**Collision to resolve**: command 0x60 is Heartbeat in `N2S_COMMAND` and GyroParamGet/Set/Reset in the export table (§4.3). The two uses go through different transports (N and E). Which one the device accepts for 0x60 is not established. Do not assume.

### 4.3 Extra exports: addresses and frames

Export addresses from the DLL's export table (`observed-static`). Frame bytes from the export code, with `derived-static` checksums. The exchange column is the `n, n` argument pair of `0x1004a820`.

| Export | Address | Frame (hex) | Exchange | Reply / output | Label |
|---|---|---|---|---|---|
| ScreenBrightnessSet | 0x407a0 | `00 43 01 05 b cs` | 5,5 | none (returns 0) | observed-static; cs `derived-static` |
| ScreenBrightnessGet | 0x40510 | `00 43 00 05 00 48` | 5,5 | value from buf[3] | frame derived; reply offset `inferred` |
| ScreenLanguageSet | 0x40f40 | `00 44 01 05 lang cs` (lang = buf[4]) | 5,5 | none | observed-static; cs `derived-static` |
| ScreenLanguageGet | 0x40a20 | `00 44 00 05 00 49` | 5,5 | buf[3] | frame derived; reply `inferred` |
| ScreenLanguageGetDefault | 0x40cb0 | `00 44 02 05 00 cs` | 5,5 | buf[3] | `inferred` (cs placement) |
| ScreenLanguageSetDefault | 0x411c0 | `00 44 03 05 lang cs` | 5,5 | none | address observed-static; layout `inferred` |
| ScreenSleepTimeGet | 0x41440 | `00 45 00 08 [00 00 00 00] cs` (cs at byte 8) | 8,8 | u32 big-endian from buf[3..6] | payload zeros `inferred`; time unit unknown |
| ScreenSleepTimeSet | 0x41710 | `00 45 01 08 t3 t2 t1 t0 cs` (big-endian u32 at buf[4..7]) | 8,8 | none | address observed-static; cs `derived-static` |
| KeyRapidFireGet | 0x3fe60 | `00 F7 00 08 … cs` (cs at byte 8) | 8,8 | three flag bits in the reply | payload positions `inferred` |
| KeyRapidFireSet | 0x40220 | `00 F7 01 08 … cs` | 8,8 | none | six flags packed into one byte; byte placement `inferred` |
| GyroParamGet | 0x3f510 | `00 60 00 1e` + buf[4] from caller, buf[5..29] zero, cs at byte 30 | 0x1e,0x1e | `p[k] = buf[k+3]`, k = 0..4 (observed-static write-back) | cs = (sum of bytes 1..29) & 0xFF, `derived-static`; constant 0x7E = 0x60+0x1E+0x00 (observed-static sums). Whether byte 30 is on the wire (write length 0x1e) is `inferred` |
| GyroParamSet | 0x3fad0 | `00 60 01 1e`; buf[22..29] = 8 payload bytes from caller struct (observed-static); buf[4..21] source not read; cs at byte 30 | 0x1e,0x1e | none | cs = (sum of bytes 1..29) & 0xFF, `derived-static`; constant 0x7F = 0x60+0x1E+0x01 (observed-static) |
| GyroParamReset | 0x3f8e0 | `00 60 02 04 66` | 4,4 | none | observed-static; cs `derived-static` (0x60+0x02+0x04 = 0x66) |
| StandLightGet | 0x42a20 | `00 61 00 0a [6 payload bytes] cs` (cs at byte 10) | 10,10 | `*p = buf[3]`, `p[1] = buf[4]`, `p[2] = buf[5]` | frame layout `derived-static`; reply `inferred` |
| StandLightSet | 0x42ed0 | `00 61 01 0a` + payload at buf[4..9] + cs at byte 10 | 10,10 | none | one payload input not mapped (`inferred`); cs = sum(1..9) `derived-static` |
| StandLightReset | 0x42ce0 | `00 61 02 04 67` | 4,4 | none | observed-static; cs `derived-static` (0x61+0x02+0x04 = 0x67) |

Not N2S-only (present in the family, shared with other models, excluded here): `ScreenThemeColorGet/Set` (0x419a0 / 0x41c80), `ScreenWallpaperGet/Set` (0x41f80 / 0x425e0).

Context exports (not frame-analysed here): `SAPJoyUniDevice_SetGyroscopeCalibration` 0x48480, `…GetGyroscopeCalibration` 0x47090, `…SetGamepadStateReport` 0x47f90, `…GetConfigPage` 0x46b10, `…GetCustomCalibration` 0x3b750.

Worked checksum examples (`derived-static`): GyroParamReset `00 60 02 04 66`; StandLightReset `00 61 02 04 67`; ScreenBrightnessGet `00 43 00 05 00 48`; ScreenLanguageGet `00 44 00 05 00 49`; SetGamepadStateReport `00 F3 01 05 m cs` with `cs = (F3+01+05+m) & 0xFF`.

### 4.4 Where the app reaches the N2S extras

App structure (`observed-static`):

- The app's T4N2 binding wraps each export as a thin pass-through: call the export, return success when the result equals `StatusSuccess`, log `[T4N2]<Name> success/failed`. Getters allocate and return the value (e.g. ScreenBrightnessGet, ScreenLanguageGet, ScreenSleepTimeGet).
- A shared device-model object holds the screen and stand-light methods:
  - `getMotionInfo` → `GyroParamGet(1)`, `GyroParamGet(0)`.
  - `setMotionInfo` → `GyroParamSet(e)` (two call sites).
  - `resetMotionInfo` → `GyroParamReset()`.
  - `setSecreenInfo` → `ScreenThemeColorSet` (if set), `ScreenSleepTimeSet`, `ScreenBrightnessSet`.
  - `getScreenInfo` → `ScreenThemeColorGet()`, `ScreenBrightnessGet()`, `ScreenSleepTimeGet()`.
  - `getStandLightInfo` → `StandLightGet()`. `setStandLightInfo` → `StandLightSet`. `resetStandLightInfo` → `StandLightReset()`.
- The N2S device model is that object plus N2S-specific methods (`setKeyPrfoileN2SAsync`, `setStickInfoN2SAsync`, `getMotionInfoN2SAsync`, `setMotionInfoN2SAsync`, `setGamepadInfo`, `setBasicInfo`, `setDPadInfo`, `setTriggerInfo`), with `N2S_COMMAND` and `N2S_ACTION` attached. `observed-static`.
- The N2S device class (constructor logs `proxyN2S created`) calls `getStandLightInfo` and `getScreenInfo` from `loadGlobalScreenPayload`. A second call site assigns the result of `getStandLightInfo` to the profile's light data. `observed-static` for the calls. That `loadGlobalScreenPayload` belongs to the N2S class is `inferred`.

Extras with no call site found in the app: `ScreenLanguage*` (bound and wrapped only), `KeyRapidFire*`, `ScreenSleepTime` beyond `setSecreenInfo`/`getScreenInfo`. `observed-static` (search), not proven absent for the T4N2 PIDs.

## 5. Wireless reachability and the app's device watch

### 5.1 What the app sees

- **Device table**. Both T3XS entries: `ConfigModeVid: 13623`, `ConfigModePid` 4186 / 4286, `GamepadUsagePage: 65535`, `DeviceTypeId: 0`, **`HandleByDll: true`**, `ConfigMethods: []`, and the T3XS device class as proxy. `observed-static`.
- **The T4N2 wireless record** (`AssetsFolder: "T4N2/White"`, `ConfigModePid: 4248` = 0x1098) also has `HandleByDll: true`. `observed-static`.
- **`isSupportedDevice`**: returns false for any entry with `HandleByDll === true`. The T3XS PIDs and 0x1098 therefore return false. `observed-static`.
- **`isSupportedGamepad`**: matches `GamepadUsagePage`. It has no `HandleByDll` exclusion. T3XS's `GamepadUsagePage` is 65535, which never matches a real usage page. `observed-static` for the table; the match outcome is `derived-static`.
- **A fixed PID list** in the change handler: `[4112, 4201, 4209, 4211, 4213, 4215, 4239, 4196, 4197, 4217]`. Neither 0x105A nor 0x10BE is in it. `observed-static`.

### 5.2 Config-method watch list and the firmware predicate

- The app concatenates every device's `Modes` and every device's `ConfigMethods`; `getAllMethods` returns the latter, and the change handler uses it as its watch list. `observed-static`.
- Since T3XS `ConfigMethods` is `[]`, **the T3XS entries contribute nothing to the watch list**. `derived-static`.
- The app's firmware predicate that was suspected of gating device presence is the T3Pro library's `IsFirmwareUpdateSupported(e)`. It is **not** a device enumeration predicate and says nothing about device presence. `observed-static`.

### 5.3 Device-change callback

- The DLL device-change callback logs `DeviceChangeCallback error` on exception. `observed-static`. Its body is not read here.
- The change handler: on add, if the watch list contains (Pid, Vid), fire after 300 ms. If `isSupportedDevice` is true, connect with retries (150 ms, up to 5). T3XS and 0x1098 satisfy neither. `observed-static`.
- On remove, the controller-removal path runs for cached HID devices. The T3XS path is not on it. `observed-static`.

### 5.4 DLL watch list

- The app loads the T3Pro library `sapjoyuniapis_x86_v1.0.16` and the T4N2 library `sapjoyuniapis_t4n2_x86_v2606151` from its dfu folder. `observed-static`.
- Init returns a closure that creates a SAPJoy instance with `pcwszName: "GameSirConnect"`, `vid: [13623]`, and a PID list. The **PID list was not recovered**. `inferred`: it is the set of PIDs the DLL watches under VID 0x3537. Its contents are the open question for wireless reachability.
- VID 0x413E: no reference in the app (search empty). It exists only in the DLL (`0x100345d0`, `T3XS_Execute` step 2). `observed-static`.

### 5.5 Bounded answer for each wireless PID

| PID | Interface type | Who opens it | Bound |
|---|---|---|---|
| 0x1098 (T4N2 wireless ConfigMode) | VID 0x3537 usage 0xFF7A | T4N2 exchange `0x1004a520` (usage-only, `inferred`) | **Reachable by the DLL if the OS exposes the node.** App table record `HandleByDll: true`, so `isSupportedDevice` is false; the app does not connect it through `isSupportedDevice`. Whether the DLL's PID list (§5.4) includes 0x1098 is not recovered. |
| 0x10BD (N2S wireless) | VID 0x3537 usage 0xFF7A | `N2S_Execute` probe `0x10030580(0x3537, 0x10BD, -0x86)` and open fallback | **Reachable by PID filter in code.** Device exists in the table (`ConfigModePid` 4285). |
| 0x10BE (T3XS wireless) | VID 0x3537 usage 0xFF7A | T3XS path A/B selector (usage-only; no PID) | **Reachable by code if the OS exposes the node.** Not in the fixed PID list, not in the watch list, `isSupportedDevice` false; only the DLL can open it. Whether the DLL's PID list (§5.4) includes 0x10BE is not recovered. |

What is not answered: whether a real wireless receiver exposes usage 0xFF7A on VID 0x3537 with PID 0x10BE. No hardware was opened, per the task rules. The code bound is: the DLL's usage-only T3XS selector would accept it; the app table would not.

## 6. Dispatch contradiction (open)

- The T4N2 frame branch in `0x10047f90` runs only when both T3XS selectors return 0 (§2.6).
- Both T3XS selectors match any VID 0x3537 node with usage 0xFF7A, with no PID check (§1).
- The T4N2 family uses the same VID and usage page (`sapjoy-t4n2.md` §2). The T4N2 exchange `0x1004a520` is also usage-only (§4.1, `inferred`).
- So a T4N2 device that exposes a usage-0xFF7A node on VID 0x3537 would make the T3XS selectors nonzero. `SetGamepadStateReport` would then take path A or B and send the T3XS frame (tag `C0`, cmd 0x1C), not the T4N2 frame `00 F3 01 05 m cs`.
- Three readings are possible, and none is confirmed: (a) T4N2 devices expose a different usage or interface than the selectors see; (b) the selectors' enumeration differs in a way not yet read; (c) the T4N2 frame path is not used on those devices at all. Hardware observation would settle it; none was made.
- `T3XS_Execute` step 1 routes VID 0x3537 usage 0xFF7A to path A. Step 2 routes VID 0x413E to path B. The app table gives ConfigModeVid 13623 (0x3537) for both T3XS PIDs, which fits step 1 and the selectors, not step 2.
- `derived-static`: the T3XS path does not require a PID filter because usage page 0xFF7A on VID 0x3537 is the only discriminator the DLL uses. Any other 0x3537 device with usage 0xFF7A would be routed to T3XS too. Unverified.

## 7. Not examined or unresolved

- The T4N2/usage-0xFF7A overlap in §6: not resolved.
- `0x10031810` ("Send:" debug log path): not read. Assumed log-only; not confirmed.
- GyroParamGet/Set checksum: resolved (§4.3). Still open: source of buf[4..21] in GyroParamSet (most of 0x1003fad0 not read); whether byte 30 is sent (write length 0x1e).
- `0x1000d220` (mod_t3xs write) and `0x1000ceb0` (read) bodies: not read. Call shapes only (§3.3).
- mod_t3xs checksum formula, reply mapping, and the filter globals at 0x1004b034 / 0x1004b038: not decoded.
- mod_t3xs loader: not located.
- PID list in the app's SAPJoy `init` closure: not recovered.
- `0x1004a520` PID filter: not read.
- `ScreenBrightnessGet` / `ScreenLanguageGet` reply byte placement: `inferred` from stack-slot use.
- `StandLightSet` input mapping for one payload slot: not visible.
- ScreenSleepTime time unit: not established.
- N2S `loadGamepadInfo` and `setKeyPrfoileN2SAsync` internals: not read (out of scope of N2S extras).
- App-side facts in §1, §2.5, §4.4 and §5 that were not re-checked in this pass: `inferred` until checked.
- Changes that would confirm any of the above need the DLL disassembly at the listed addresses, not device access.

## 8. Cross-references

- Selector, exchange helper, GetVersion, 64-byte frame: `sapjoy-t4n2.md` §2-3. This file corrects its T3XS PID implication (§1 above).
- T4N2 config, page and stick frames: `sapjoy-t4n2-config.md`.
- Items in `sapjoy-t4n2.md` §8 "Unresolved" that this file addresses: T3XS transfer (§2), N2S_Execute and N2S extras (§4), wireless PID reachability (§5). Items still open are in §7.
