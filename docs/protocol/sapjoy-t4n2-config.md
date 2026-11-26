# SAPJoy T4N2 configuration exports: frame map

Scope: configuration exports of `sapjoyuniapis_t4n2_x86_v2606151.dll` (HID path used by T4N2 / N2S).
Companion to `docs/protocol/sapjoy-t4n2.md`. That file owns the exchange helper (its section 3), the GS_CMD list (section 6), the 38-byte input layout (section 7) and the open items (section 8). This file does not repeat them.

Labels: `observed-static` (observed in the vendor library's code, address cited), `derived-static` (arithmetic over observed bytes), `inferred` (role or branch guessed from context).

Addresses: VA = 0x10000000 + RVA. Export map: the library's export table (78 entries; one fewer distinct VA, the 78 vs 77 gap is not explained).

Status: the export-to-helper link is complete for the 48 exports that call a keyed helper (48 rows in section 3). Frame bodies for most exports are not yet read. No hardware was used.

---

## 1. Dispatch and device path

- **Family predicate.** `0x10035f70` (`observed-static`, 0x10035f70-0x10035fb8) looks up the device-list entry tagged `0x3537` and returns 1 if a node has usage page `0xFF7A` (stored as `-0x86`). This is the same selector as `sapjoy-t4n2.md` §1.
- **Other family predicate.** `0x10033af0` (`observed-static`) is called before `0x10035f70` in SetGamepadStateReport (0x10047f90), inside a guard that a global state is not 0x7fffffff. Its body is not read.
- **Example branch, SetGamepadStateReport** (`0x10047f90`, `observed-static`):
  - `0x10033af0()` false and `0x10035f70()` true → `0x10035720(mode)`, which is the A5 keyed path with key `0x1C` (section 3).
  - Both false → opens the device with `0x1004a520()`, builds the F3 frame (section 4.1), exchanges with `0x1004a820(…, 0x40, 5, 5)`.
  - `0x10033af0()` true → `0x10032d20()`. Not read.
- **Which branch each export takes.** Only SetGamepadStateReport is read at branch level. For the other exports, the A5 link in section 3 is the keyed call found in the export body (`observed-static`), and its branch is `inferred` from the predicate call.

## 2. Frame rules

- **Buffer.** 64 bytes, zero-filled with `memset(…,0,0x40)` before each build. Byte 0 is the report id, 0x00 (`observed-static`, same as `sapjoy-t4n2.md` §3).
- **Two framings.**
  - **A5 keyed frame** (section 3): byte 1 = `0xA5`, byte 2 = `L`, byte 3 = `0xC0`, byte 4 = key.
  - **F-family frame** (section 4): 16-bit command at bytes 1-2 (little-endian: command 0x1F1 gives `F1 01`), byte 3 = `L`.
- **Checksum rule (both framings).** The checksum sits at index `L` and equals the 8-bit sum of bytes 1 to `L-1` (`derived-static`).
  - Confirmed on: `46 01 04 4B` (`0x100423F0`), `60 02 04 66` (`0x1003F8E0`), `61 02 04 67` (`0x10042CE0`) (`observed-static`; checksum = sum of bytes 1..3 at index 4); the F3 frame in section 4.1 (sum of bytes 1-4 at index 5, `observed-static`, in `0x10047f90`). The A5 layout in section 3 uses the same rule as `inferred`, not yet checked on a helper write.
  - The 9-byte and 0x11-byte F-family frames use the same rule; `inferred` (the checksum buffer offset was not confirmed).
- **Exchange call.** `0x1004a820(handle, &buf, 0x40, a, b)`. In most frames `a == b == frame length`. Exceptions: GetVersion `(3,0x20)`, GetMacroStep `(6,0x10)`, SetABXYRotationTest `(3,3)`, GetCustomCalibration/SetCustomCalibration `(6,7)`. Reading of the pair as retry/timeout comes from `sapjoy-t4n2.md` §3; `inferred` for the others.
- **Reply.** Reply lands in the same 64-byte buffer. Reply byte 3 is the first data byte (`observed-static`, e.g. at `0x1002bf90` and `0x1003e7c0`). Single-value replies pass through `0x1004af50` (reply-code to value; role `inferred`).
- **Key argument.** Stick/key arguments pass through `0x1004af00(arg)` into byte 4 in the F-family (`observed-static`, `0x1003eaf0`; role `inferred`).
- **SetABXYRotationTest exception.** Frame is `[1]=1E, [2]=v, [3]=v+1E` (checksum, not length). Section 2's "byte 3 = L" does not hold here (`observed-static`, `0x1003cd20`).

## 3. A5 keyed family

Generic keyed sender: `0x10035ca0(key, payload_ptr, payload_len, out_buf, 0x40)` → `0x10034620` builds the frame and calls `0x1004a820`. Timeout 2000 ms in `0x10035ca0` (`observed-static`).

Frame layout (`0x10034620`, `observed-static`):

| Byte index | Value |
|---|---|
| 0 | 0x00 (report id) |
| 1 | 0xA5 |
| 2 | `L` = payload_len + 5 |
| 3 | 0xC0 |
| 4 | key |
| 5 … 4+payload_len | payload |
| `L` (= payload_len + 5) | checksum: sum of bytes 1..L-1, mod 256 (`inferred` for the exact range; the code folds a byte sum with 16-bit pair accumulation) |

Written frame length = payload_len + 6.

Worked example, GetLightParam (key 0x0F, payload 0): bytes `00 A5 05 C0 0F 79`, length 6. Checksum = (A5+05+C0+0F) mod 256 = 0x79 (`derived-static`).

Keyed exports (caller → key, `observed-static` from the export body; key and payload length from the helper body):

| Export | VA | Helper | Key | Payload | Output |
|---|---|---|---|---|---|
| SAPJoyUniDevice_GetCustomCalibration | 0x1003B750 | 0x10034C60 | 0x05 | 0 | 2 bytes |
| SAPJoyUniDevice_GetDeviceInfo | 0x1003BA10 | 0x10034D20 | 0x01 | 0 | 2 bytes |
| SAPJoyUniDevice_GetMacAddress | 0x1003C0B0 | 0x10035000 | 0x02 | 0 | 3 bytes |
| SAPJoyUniDevice_GetVersion | 0x1003C310 | 0x10035390 | 0x03 | 0 | 4 bytes |
| SAPJoyUniDevice_SetCustomCalibration | 0x1003C570 | 0x100355E0 | 0x04 | 2 | none |
| SAPJoyUniDevice_GetDPadParam | 0x1003C850 | 0x10034CC0 | 0x07 | 0 | 1 byte |
| SAPJoyUniDevice_GetKeyMappingHSV | 0x1003CB40 | 0x10034EE0 | 0x0A | 1 | out |
| SAPJoyUniDevice_SetABXYRotationTest | 0x1003CD20 | 0x10035540 | 0x08 | 1 | none |
| SAPJoyUniDevice_SetDPadParam | 0x1003CF60 | 0x10035640 | 0x06 | 4 | none |
| SAPJoyUniDevice_SetKeyMappingHSV | 0x1003D420 | 0x10035800 | 0x09 | 0x11 | none |
| SAPJoyUniDevice_GetLightColorHSV | 0x1003D800 | 0x10034F40 | 0x0F | 0x11 | out |
| SAPJoyUniDevice_GetLightParam | 0x1003DC00 | 0x10034FA0 | 0x0F | 0 | 3 bytes |
| SAPJoyUniDevice_SetLightColorHSV | 0x1003DDD0 | 0x10035850 | 0x0E | 0x11 | none |
| SAPJoyUniDevice_SetLightColorTest | 0x1003E3B0 | 0x100358A0 | 0x0B | 4 | none |
| SAPJoyUniDevice_SetLightParam | 0x1003E580 | 0x100358F0 | 0x0C | 3 | none |
| SAPJoyUniDevice_GetMacroStep | 0x1003E7C0 | 0x10035060 | 0x12 | 2 | out |
| SAPJoyUniDevice_RecordMacro | 0x1003EAF0 | 0x10035460 | 0x10 | 2 | none |
| SAPJoyUniDevice_SetMacroStep | 0x1003EDE0 | 0x10035940 | 0x11 | 0x10 | none |
| SAPJoyUniDevice_GetStickCurveHSV | 0x10043350 | 0x10035200 | 0x14 | 1 | out |
| SAPJoyUniDevice_GetStickMapping | 0x100436E0 | 0x10035270 | 0x16 | 1 | out |
| SAPJoyUniDevice_SetStickCurveHSV | 0x100438D0 | 0x10035AC0 | 0x13 | 0x16 | none |
| SAPJoyUniDevice_SetStickMapping | 0x10043EA0 | 0x10035B10 | 0x15 | 0x0C | none |
| SAPJoyUniDevice_GetTriggerParam | 0x10044210 | 0x10035330 | 0x18 | 1 | out |
| SAPJoyUniDevice_SetTriggerParam | 0x100443B0 | 0x10035BB0 | 0x17 | 0x10 | none |
| SAPJoyUniDevice_GetVibrateLevel | 0x10045D90 | 0x10035400 | 0x1B | 0 | 2 bytes |
| SAPJoyUniDevice_SetVibrateLevel | 0x10046080 | 0x10035C00 | 0x1A | 6 | none |
| SAPJoyUniDevice_SetVibrateTest | 0x10046430 | 0x10035C50 | 0x19 | 4 | none |
| SAPJoyUniDevice_GetConfigPage | 0x10046B10 | 0x10034C00 | 0x22 | 0 | 1 byte |
| SAPJoyUniDevice_GetDeviceType | 0x10046C70 | 0x10034DA0 | 0x26 | 0 | 1 byte |
| SAPJoyUniDevice_GetGamepadParam | 0x10046E10 | 0x10034E00 | 0x2E | 0 | out |
| SAPJoyUniDevice_GetGyroscopeCalibration | 0x10047090 | 0x10034E60 | 0x2A | 0 | out |
| SAPJoyUniDevice_GetMotionSenseCurveHSV | 0x100471F0 | 0x100350D0 | 0x1E | 0 | 2 bytes |
| SAPJoyUniDevice_GetMotionSenseMapping | 0x10047360 | 0x10035130 | 0x20 | 0 | 2 bytes |
| SAPJoyUniDevice_GetReportRate | 0x100474C0 | 0x100351A0 | 0x30 | 0 | 1 byte |
| SAPJoyUniDevice_GetStickTriggerCalibration | 0x10047650 | 0x100352D0 | 0x28 | 0 | 1 byte |
| SAPJoyUniDevice_ResetConfigPage | 0x100477C0 | 0x100354C0 | 0x24 | 0 | none |
| SAPJoyUniDevice_SaveConfigPage | 0x10047920 | 0x10035500 | 0x23 | 0 | none |
| SAPJoyUniDevice_SetConfigPage | 0x10047A80 | 0x10035590 | 0x21 | 1 | none |
| SAPJoyUniDevice_SetDeviceType | 0x10047BE0 | 0x10035690 | 0x25 | 0 | none |
| SAPJoyUniDevice_SetGamepadParam | 0x10047D40 | 0x100356D0 | 0x2D | 9 | none |
| SAPJoyUniDevice_SetGamepadStateReport | 0x10047F90 | 0x10035720 | 0x1C | 1 | none |
| SAPJoyUniDevice_SetGyroscopeCalibration | 0x10048480 | 0x10035770 | 0x29 | 1 | none |
| SAPJoyUniDevice_SetISPMode | 0x100485E0 | 0x100357C0 | 0x2B | 0 | none |
| SAPJoyUniDevice_SetMotionSenseCurveHSV | 0x10048740 | 0x10035990 | 0x1D | 0x14 | none |
| SAPJoyUniDevice_SetMotionSenseMapping | 0x100488B0 | 0x100359E0 | 0x1F | 0x0B | none |
| SAPJoyUniDevice_SetPairMode | 0x10048A10 | 0x10035A30 | 0x2C | 0 | none |
| SAPJoyUniDevice_SetReportRate | 0x10048B70 | 0x10035A70 | 0x2F | 2 | none |
| SAPJoyUniDevice_SetStickTriggerCalibration | 0x10048D00 | 0x10035B60 | 0x27 | 1 | none |

Notes on the table:
- Output size comes from the helper body (`observed-static`). "out" means output size not yet read.
- Key `0x0F` serves two exports with different payload: GetLightParam (payload 0) and GetLightColorHSV (payload 0x11). Both `observed-static`.
- Key-to-export pairs come from the export body call, so the link is `observed-static`. The branch on `0x10035f70` is `inferred`.
- SetGamepadStateReport key `0x1C` payload 1 is the mode byte. Its frame would be `00 A5 06 C0 1C mode ck` (payload 1, L=6, checksum at index 6). Derived from the layout, not read from a frame write (`inferred`).

## 4. F-family frames

### 4.1 F3 frame, SetGamepadStateReport fallback (`observed-static`, `0x10047f90`)
- Bytes: `[1]=F3`, `[2]=01`, `[3]=05`, `[4]=mode`, `[5]=checksum` (sum of bytes 1-4). Exchange `(5,5)`. Only on the branch where `0x10033af0` and `0x10035f70` are both false.

### 4.2 Export-owned F-family frames (`observed-static`, builder VA equals the export VA)
These builders are the export entry points themselves, so the owner is resolved.

| Cmd (1,2) | L | Exchange | Export | VA | Note |
|---|---|---|---|---|---|
| 60 02 | 4 | 4,4 | SAPJoyUniDevice_GyroParamReset | 0x1003F8E0 | checksum 0x66 |
| 46 01 | 4 | 4,4 | ScreenWallpaperReset | 0x100423F0 | checksum 0x4B |
| 61 02 | 4 | 4,4 | StandLightReset | 0x10042CE0 | checksum 0x67; resolves the open item |
| F7 00 | 8 | 8,8 | KeyRapidFireGet | 0x1003FE60 | reply bits 0-2 of byte 4; same cmd as GetCustomCalibration in 4.3 (L=6): collision, unresolved |
| F7 01 | 8 | 8,8 | KeyRapidFireSet | 0x10040220 | same cmd as SetCustomCalibration in 4.3 (L=6): collision, unresolved |
| 43 01 | 5 | 5,5 | ScreenBrightnessSet | 0x100407A0 | |
| 44 01 | 5 | 5,5 | ScreenLanguageSet | 0x10040F40 | |
| 44 03 | 5 | 5,5 | ScreenLanguageSetDefault | 0x100411C0 | |
| 45 01 | 8 | 8,8 | ScreenSleepTimeSet | 0x10041710 | |
| 42 01 | 8 | 8,8 | ScreenThemeColorSet | 0x10041C80 | |
| 47 00 | 7 | 5,? | ScreenWallpaperGet | 0x10041F80 | exchange second argument not read |
| 41 01 | loop | iVar20 | ScreenWallpaperSet | 0x100425E0 | chunked |
| 61 00 | 0x0A | 0xA,0xA | StandLightGet | 0x10042A20 | reply bytes 3..5 |
| 61 01 | 0x0A | 0xA,0xA | StandLightSet | 0x10042ED0 | |
| 60 ?? | 0x1E | 0x1E,0x1E | GyroParamGet | 0x1003F510 | reply into param_1[0..2]; cmd bytes 1-2 not captured |

### 4.3 F-family frames with owner not resolved
These builders are not export entry points. Their callers were not traced.
- F0 00, L=0x11, `0x10043150` (reply into param_3+10..12) and `0x100305D0` (reply into param_3+4..6). Paired with GetStickCurveHSV (0x10043350) by `inferred` only.
- F0 01, L=0x11, `0x10043250` (page 1 or 2, called from SetStickCurveHSV 0x100438D0, `observed-static`) and `0x1002E8C0` (page 3 or 4, called from `0x1002C740`, `observed-static`).
- F1 00, L=9, `0x1002BF90`; L=0x0B, `0x1002F790`. Get key mapping. Called via `0x1004ab70` / `0x1004abd0` (`inferred`). Reply parse at `0x1002BF90` (`observed-static`): byte 3 → output +7; byte 4 → count → output +0x0C; byte 5 → output +0x0D; byte 5 == 0xD2 sets output +0x0B to 0, otherwise 1; byte 6 → +0x0E (if count > 1); byte 7 → +0x0F (if count > 2).
- F1 01, L=9, `0x1002C4D0`; L=0x0B, `0x1002FD10`. Set key mapping variants.
- F9 00, L=6, `0x1002F9A0` (one-word reply); F9 01, L=6, `0x1002FFA0`.
- F8 00, L=8, `0x1002FB40` (reply bytes 3,4,5 into param_3); F8 01, L=8, `0x10030130`.
- F5 00 / F5 01, L=6: builders `0x10046E10` (F5 00) and `0x10047D40` (F5 01), cmd bytes `observed-static` from the builder table. `0x10046E10` is the GetGamepadParam export entry, but its body also calls the keyed helper in section 3. Which path a call takes is unresolved (`inferred`). Owner of `0x10047D40` not confirmed.
- F6 00..04: GetMacroStep / SetMacroStep / RecordMacro bodies use the keyed helper in section 3 (`F6 00` L=6 exchange `(6,0x10)`, `F6 01` L=0x10, `F6 02` L=5, `F6 03` L=0x0A, `F6 04` L=5). Their write offsets were read (`observed-static`, lines 99-125) but the export-to-branch link is `inferred`.
- F7 00 / F7 01, L=6: GetCustomCalibration `0x1003B750` / SetCustomCalibration `0x1003C570`, exchange `(6,7)`. Reply: byte 3 → `*param_2`, buffer word → `*param_3` (`observed-static`, lines 97-98). Cmd bytes collide with the KeyRapidFire rows in 4.2 (L=8); unresolved.
- 23 01 06 01 and 23 04 06 01 (`0x1003CF60`, SetDPadParam): two frames written inline (`observed-static`, lines 139, 201). L=6, exchange `(6,6)`. Byte 4 (`01`) is a mode value; meaning `inferred`. The SetDPadParam body also calls the keyed helper `0x10035640` (key 0x06). Which branch sends which frame is `inferred`.
- 1E ?? L=3: SetABXYRotationTest (section 2).
- 0x10045D90 (GetVibrateLevel) and 0x10046080 (SetVibrateLevel) keyed in section 3 but F-family builders `0x10045D90`/`0x10046080` also show L=0x0D. Cmd bytes not captured. Treat as unresolved.

## 5. Internal constants (no HID exchange)
- SAPJoyUniDevice_GetDevicePID (0x1003BC80) → helper `0x10034D90` writes `0xBE10` into its output (`observed-static`, `0x10034d90` line 16). No frame.
- SAPJoyUniDevice_GetHardwareID (0x1003BEA0) → helper `0x10034EC0` writes `0x10BE` and `0x3537` (`observed-static`, `0x10034ec0` lines 16-17). No frame. `0x3537` is the lookup tag used by the predicate in section 1.

## 6. Exports with no resolved frame or key yet
- Dispatch only, no keyed call in the export body (their branch helpers are not read): GyroParamGet (0x1003F510; builder is itself, section 4.2), GyroParamReset (0x1003F8E0; 4.2), GyroParamSet (0x1003FAD0), KeyRapidFireGet/Set (4.2), ScreenBrightnessGet (0x10040510), ScreenBrightnessSet (4.2), ScreenLanguageGet (0x10040A20), ScreenLanguageGetDefault (0x10040CB0), ScreenLanguageSet/SetDefault (4.2), ScreenSleepTimeGet (0x10041440), ScreenSleepTimeSet (4.2), ScreenThemeColorGet (0x100419A0), ScreenThemeColorSet (4.2), ScreenWallpaperGet/Reset/Set (4.2), StandLightGet/Reset/Set (4.2).
- N2S_Execute (0x10031BB0): no helper in the call map. Not read.
- T3XS_Execute (0x10034260): calls `0x100345D0`, `0x10034620`, `0x10035F70`. Not read.
- Keyed exports: the export-to-helper link is complete for the 48 keyed exports (rows in section 3).

## 7. 38-byte input report (SAPJOYUNI_GAMEPADSTATE)
Not checked in this pass. Compare against `sapjoy-t4n2.md` §7 when the parse is found.

## 8. Conflicts with `sapjoy-t4n2.md` (for Main)
- GetVersion. `sapjoy-t4n2.md` §3 gives its request as `[1]=0x01,[2]=0x01`. This file: the export `0x1003C310` calls the A5 helper with key `0x03`, empty payload (`observed-static`). Both may hold on different branches (predicate true → A5; false → F-family), but this is unresolved (`inferred`).
- SetGamepadStateReport. `sapjoy-t4n2.md` §3 gives `F3 01 05 mode` with checksum at `[5]`. That frame is the fallback branch (section 4.1). When `0x10035f70` is true, the export sends the A5 frame with key `0x1C` (section 3, `observed-static`). Both are real.

## 9. Open items
- Read the branch bodies of `0x10033af0` and `0x10032d20` to confirm the other-family path.
- Confirm the GetVersion conflict (section 8) by reading the rest of `0x1003C310`.
- Read the cmd bytes for GetVibrateLevel, SetVibrateLevel, GetDPadParam, GetStickCurveHSV, GetLightColorHSV, SetLightColorHSV, GyroParamGet (1-2).
- Confirm the checksum index for the F-family 9- and 0x11-byte frames.
- Resolve `0x1004af00` (key id) and `0x1004af50` (reply code). Roles are `inferred` only.
- Read N2S_Execute (0x10031BB0) and T3XS_Execute (0x10034260).
- Locate the input parse for section 7.
- The vendor app's argument fill was not read. Keys for `0x10035A30`-`0x10035C50` and `0x100436E0`-`0x10048D00` not confirmed by search.

Skipped / not checked: no hardware, no wire capture, static reading only; app argument fill not verified; branch selection for all exports except SetGamepadStateReport is `inferred`; checksum index for 9- and 0x11-byte frames inferred; N2S_Execute and T3XS_Execute not read. Risk: do not write to a device from any frame or key marked `inferred` or from the GetVersion / SetGamepadStateReport conflicts in section 8 until the branch is confirmed.
