# GameSir Tarantula CE (T3CE) profile codec

Verified layout, implementation and tests for the profile and light blocks: `hid-family-profile.md` (`layouts/t3ce-*.json`, `t3ce-raw-vectors.json`). This page adds T3CE details (touch keys, packet builders) observed from the vendor app's behaviour.

Byte-level spec of the T3CE profile block (1935 bytes) and light block (102 bytes)
that the vendor app encodes and decodes. Golden vectors are in
`t3ce-vectors.json` (same directory).

Scope: profile and light block layout, field semantics, and the packet builders
that carry them. Transport sequencing beyond the builders, firmware, DFU and UI are
out of scope. No hardware evidence is used in this document.

Labels: `observed-static` (observed from the vendor app), `vendor-exec`
(vector produced by running the vendor encoder/decoder), `inferred`, and
`unresolved` (named, not decided). Unknown ranges are marked as such.

## 0. Model identity

`observed-static`. Wired PID 0x103D (4157), UsagePage 65520,
pure HID (no SAPJoy DLL). A second wired PID 0x090D (2317) appears with a restricted
feature set (no KeyboardAndMouse, MapToMoreKeys, Turbo disabled; HiddenMenus
`/motion`). This document treats 0x090D as the same protocol. `unresolved`: whether
0x090D's profile length differs. The vendor values in section 1 apply to both.

`observed-static`, vendor constants for T3CE:

| Name | Value |
|------|------:|
| EveryPacketStartIndex | 5 |
| EveryPacketLength (profile) | 1935 |
| LightProfilePacketLength | 102 |
| GamepadKeyStateReportId | 18 |
| EveryOutPacketLength | 58 |
| ShiftProfileNumber | 5 |
| LightProfileNumber | 32 |
| CurrentOfProfileNumber | 48 |
| MacroStepMaximum | 32 |
| AnimationPacketLength | 20 |
| RGBCount | 5 |

## 1. Profile block

A profile is one 1935-byte buffer. `fromBytes` returns an empty profile unless the
input length is exactly 1935 (`observed-static`).

| Offset | Size | Field |
|-------:|-----:|-------|
| 0 | 32 | Name (UTF-8, trimmed on decode) |
| 32 | 32 | FunData (section 2) |
| 64 | 112 | 16 mapping blocks x 7 B (section 3), order in 3.1 |
| 176 | 1521 | 9 function-key blocks x 169 B (section 4): FL1, FR1, Ext1-7 |
| 1697 | 32 | L2 trigger block (section 5) |
| 1729 | 32 | R2 trigger block (section 5) |
| 1761 | 36 | Left stick block (section 6) |
| 1797 | 36 | Right stick block (section 6) |
| 1833 | 41 | Aim motion block (section 7) |
| 1874 | 41 | Tilt motion block (section 7) |
| 1915 | 10 | Ext8 touch-key block (section 8) |
| 1925 | 10 | Ext9 touch-key block (section 8) |
| 1935 | — | end |

Sum check: 32+32+112+1521+32+32+36+36+41+41+10+10 = 1935.

Sources: `observed-static`, decode and encode.
Block sizes from the vendor constants: Name 32, FunData 32, Mapping 7,
TouchKey 10, FunctionKey 169, Stick 36, Trigger 32, Motor 6, Audio 5, Sensor 41.

## 2. FunData (offset 32, 32 bytes)

`observed-static`. Decode order:

| Rel. | Size | Field |
|-----:|-----:|-------|
| 0 | 4 | motor left, right, lt, rt (levels) |
| 4 | 5 | audio: enable, volume, mixer, mic_mute, mic_sensitivity |
| 9 | 8 | extend: shift_en, shift_value, dpad_diagonal_lock_en, xinput_abxy_change, switch_abxy_change, report_rates_gears, sensor_quick_mode, stick_resolution |
| 17 | 15 | reserved (`unresolved`: not read by decode; encoder leaves zeros in vectors) |

App-profile conversion (`observed-static`): audio enable is bit 0 of the
audio byte; `stickResolution = 4 - stick_resolution` (`observed-static`).

## 3. Mapping blocks (offset 64, 16 x 7 B)

Button order (`observed-static`):

| Index | Button | Index | Button |
|------:|--------|------:|--------|
| 0 | Up | 8 | CRO |
| 1 | Under | 9 | CIR |
| 2 | Left | 10 | SQU |
| 3 | Right | 11 | TRI |
| 4 | L1 | 12 | PS |
| 5 | R1 | 13 | Sel |
| 6 | L3 | 14 | Sta |
| 7 | R3 | 15 | Camera |

Extended order (`observed-static`): FL1, FR1, Ext1-7.
Full key list: Ext1-7, L2, R2, Ext8, Ext9.

Key block, 7 B (`observed-static`):

| Rel. | Field |
|-----:|-------|
| 0 | turbo_en |
| 1 | turbo_speed |
| 2 | map_en |
| 3-5 | map_value1..3 |
| 6 | toggle_en |

## 4. Function-key blocks (9 x 169 B)

`observed-static` (decode), verified against `vendor-exec` (case `macro-fl1-cycle`).

| Rel. | Size | Field |
|-----:|-----:|-------|
| 0 | 2 | turbo: turbo_en, turbo_speed |
| 2 | 4 | map: map_en, map_value1..3 |
| 6 | 1 | toggle_en |
| 7 | 1 | macro flag: bit0 marco_en, bit1 marco_key_open, bit2 marco_cycle_en |
| 8 | 1 | cycle_time_h |
| 9 | 1 | cycle_time_l |
| 10 | 1 | step_num |
| 11 | 155 | 31 step slots x 5 B: step_data, hold_h, hold_l, delay_h, delay_l |
| 166 | 3 | step slot 31: step_data, hold_h, hold_l |

11 + 155 + 3 = 169. Encode: cycle_time_h = cycleTime >> 8,
cycle_time_l = cycleTime & 255, step_num = steps.length.

`observed-static`: the step loop writes 5 bytes for slots 0-30 and 3 bytes for slot 31.
`MacroStepMaximum` = 32.

## 5. Trigger blocks (L2 at 1697, R2 at 1729, 32 B each)

`observed-static`, decode and encode.
Encoder and decoder write multi-byte fields big-endian.

| Rel. | Size | Field |
|-----:|-----:|-------|
| 0 | 1 | turbo_en |
| 1 | 1 | turbo_speed |
| 2 | 1 | dead_en |
| 3 | 8 | four 2 B dead values (big-endian): front, back, anti_front, anti_back |
| 11 | 1 | map_en |
| 12-14 | 3 | map_value1..3 |
| 15 | 1 | toggle_en |
| 16 | 1 | quick_trigger status |
| 17 | 1 | quick_trigger start |
| 18 | 1 | quick_trigger end |
| 19 | 1 | linear_module_en |
| 20 | 1 | linear_status |
| 21 | 1 | linear_data |
| 22-31 | 10 | original/target pairs 1..5 (linear curve points, section 9) |

Quick-trigger status encode: 0 if disabled, else 128 | mode (`observed-static`).

## 6. Stick blocks (Left 1761, Right 1797, 36 B each)

`observed-static`. Two-byte dead values are written
low byte first (little-endian per pair): the vendor's number-to-two-bytes helper returns
`[2, 1]` (high, low) for 258 (`vendor-exec`), and the encoder reverses it.
The trigger block (section 5) is big-endian. The two encodings differ. Both are
recorded, not unified.

| Rel. | Size | Field |
|-----:|-----:|-------|
| 0 | 1 | stick_en |
| 1 | 1 | stick_square |
| 2 | 9 | dead: dead_en, then front, back, anti_front, anti_back (2 B each, low byte first) |
| 11 | 13 | linear: linear_module_en, linear_status, linear_data, original/target 1..5 |
| 24 | 12 | map: map_en, x_flip, y_flip, axis_ratio, mouse_dpi, map_index, map_cross, map_up_value, map_down_value, map_left_value, map_right_value, map_dead_value |

2 + 9 + 13 + 12 = 36.

## 7. Motion blocks (Aim 1833, Tilt 1874, 41 B each)

`observed-static`, decode and encode.
Verified against `vendor-exec` (case `aim-motion-axis`).

| Rel. | Size | Field |
|-----:|-----:|-------|
| 0 | 1 | sensor_profile_status |
| 1 | 1 | sensor_quick_key_value |
| 2 | 1 | value2 (`unresolved`: name not given in source) |
| 3 | 1 | value3 (`unresolved`: name not given in source) |
| 4 | 1 | active_axis |
| 5 | 9 | dead (same layout as section 6) |
| 14 | 13 | linear (same layout as section 6) |
| 27 | 14 | map: map_en, x_flip, y_flip, z_flip, axis_ratio, mouse_dpi, map_index, map_cross, map_up_value, map_down_value, map_left_value, map_right_value, map_dead_value, stickSensitivity |

5 + 9 + 13 + 14 = 41.

## 8. Touch-key blocks (Ext8 1915, Ext9 10 B each)

`observed-static`, big-endian.

| Rel. | Size | Field |
|-----:|-----:|-------|
| 0 | 1 | turbo_en |
| 1 | 1 | turbo_speed |
| 2 | 1 | map_en |
| 3-5 | 3 | map_value1..3 |
| 6 | 1 | toggle_en |
| 7 | 1 | triggerMode |
| 8-9 | 2 | sensitivity (big-endian) |

Ext8 sensitivity mapping (`observed-static`, confirmed `vendor-exec`):
`raw = clamp(1080 - 9 * s, 180, 1080)`. Decode is the inverse.
A raw value of 0 decodes to 100 through the clamp. That 100 is an artifact, not a default.

Key-mode defaults (`observed-static`):

| Key | sensitivity | triggerMode |
|-----|------------:|------------:|
| Ext8 | 60 | 0 |
| Ext9 | 50 | 0 |

## 9. Linear curve quantization (finding)

`observed-static`. With `round` = JavaScript `Math.round`:
decode `app = round(raw / 255 * (H - L) + L)`, with `L = round(lo / 10)` and
`H = round(hi / 10)`. Encode `raw = round((v - L) / (H - L) * 255)`.

Because the app value is an integer, raw -> app -> raw is not a fixed point. The encoder
moves some raw bytes on re-encode. This was measured (`vendor-exec`):
two passes of `fromModel(fromBytes(b))` differ in 32 bytes. Those bytes are all in the
linear curve pairs (the linear blocks of triggers, sticks and motion):

- 1721-1726, 1753-1758 (L2 and R2 linear pairs)
- 1775, 1777, 1779, 1781, 1783 (left stick)
- 1811, 1813, 1815, 1817, 1819 (right stick)
- 1850, 1852, 1854, 1856, 1858 (aim)
- 1891, 1893, 1895, 1897, 1899 (tilt)

Stable after pass 3. Consequence: any round trip through the app value is lossy for the
curve bytes. Reproduced in `default-zero-bytes` and in every linear-carrying case.

Curve point values: `unresolved`. Two observations disagree. The vendor default curve
is 25/50/75. A decoded L2 curve
reads `{x:7,y:5}, {x:5,y:10}, {x:3,y:5}`. I did not reconcile them. Do not use the
curve numbers as fact until this is resolved.

## 10. Light block (102 B)

`observed-static`.

- Bytes 0-99: 5 animations x 20 B (section 10.1).
- Byte 100, bit 0: RaiseWakeUp.
- Byte 101: StandbyTime.

### 10.1 Animation block (20 B)

`observed-static`.

| Rel. | Size | Field |
|-----:|-----:|-------|
| 0 | 1 | mode |
| 1 | 5 | brightness: singleColor, breathing, phantom, rainbow, radar |
| 6 | 4 | speed: breathing, phantom, rainbow, radar |
| 10 | 2 | hue offsets: phantom, rainbow |
| 12 | 2 | panel h (little-endian, low byte first) |
| 14 | 1 | panel s |
| 15 | 1 | panel v |
| 16 | 2 | home h (little-endian, low byte first) |
| 18 | 1 | home s |
| 19 | 1 | home v |

1 + 5 + 4 + 2 + 4 + 4 = 20.

## 11. Packet builders

`observed-static`. Lengths are `EveryOutPacketLength` = 58 per chunk.

| Packet | Bytes | Meaning |
|---------|-------|---------|
| profile write | `[15, 4, profileNo, offHi, offLo, size]` + chunk | profile write header. Chunk size = min(58, remaining). Count = ceil(len / 58) (`inferred`). Profile 1935 B -> 34 chunks; light 102 B -> 2 chunks. |
| profile read | `[15, 16, profileNo, offHi, offLo, size, ...]` | read request, 8-slot array. Slots 6-7 left unset by the vendor app (`unresolved`). |
| switch profile | `[15, 7, e]` | "Shift" maps to 5 (`ShiftProfileNumber`). |
| request 0x09 | `[15, 9]` | request (`observed-static`) |
| request 0x0b | `[15, 11]` | request (`observed-static`) |
| get RGB state | `[15, 13]` | reqGetRgbState |
| light status | `[15, 13, play?1:0, frame]` | light status |
| heartbeat | `[15, 242, e]` (e default 0) | heartbeat, written direct to HID |
| vibrate | `[15, 32, 102, 85, l, r]` | vibrate. Stopped with `l = r = 0` after `duration || 1000` ms |
| manual update | `[15, 23, 85, 136]` | manual update enter/exit (`inferred` label) |
| set stick centre | `[15, 226, n, i, 0, 32, 0...]` 64 B | setStickCenter. Meaning of `n`, `i` `unresolved`. |

Profile notify handler (`observed-static`): `t[2]` = profileNo,
offset = 256 * t[3] + t[4], length = 256 * t[5] + t[6]. On `CurrentOfProfileNumber` (48)
it re-sends request 0x0b. Otherwise it sends a profile read. `t[0]` and `t[1]` not inspected
(`unresolved`).

## 12. Report 18 (gamepad state, partial)

`observed-static`.

- A full report goes through the state decoder. Its trigger fields are `t[8]` and `t[9]`.
- An 11-byte slice `n = t[53..63]` is decoded again through the same decoder. The
  decoder's trigger fields for that call are `n[8]` and `n[9]`, but the app then sets
  `LeftTrigger = n[9]` and `RightTrigger = n[10]`, which overrides them.

Resolved: the value of `Original.LeftTrigger` is `t[62]`, and `Original.RightTrigger`
is `t[63]`. The `Mapped` record keeps `t[8]` and `t[9]` from the full-report decode. Both branches are
`observed-static`. Not hardware-verified.

Button bits: bits 3..5 of `n[7]` for FL1, FR1, M; bits 0..6 of `n[8]` for
Ext1-7. `unresolved`: the full byte map of `n[0..6]` (button
bytes) and `n[9..10]` beyond the trigger role.

## 13. Vectors

`t3ce-vectors.json`. Each case is a vendor
`fromModel` output (hex) and a `fromBytes` decode. `roundtrip` is true only when the
bytes re-encode to themselves.

| Case | Bytes | roundtrip | Notes |
|------|------:|:---------:|-------|
| default-zero-bytes | 1935 | false | Linear drift (section 9) |
| name-motors-audio-extend | 1935 | false | Linear drift |
| key-turbo-toggle-cro | 1935 | false | Linear drift |
| macro-fl1-cycle | 1935 | false | Linear drift |
| l2-deadzone-hairtrigger | 1935 | false | Linear drift |
| r2-deadzone-anti | 1935 | false | Linear drift |
| left-stick-advance-map | 1935 | false | Linear drift |
| right-stick-curve | 1935 | false | Linear drift |
| aim-motion-axis | 1935 | false | Linear drift |
| ext8-touch-sensitivity | 1935 | false | Linear drift |
| light-zero-bytes | 102 | true | Light round trip is exact |
| light-animation1-custom | 102 | true | Light round trip is exact |

Mutation byte effects (`vendor-exec`): the
name/motors/audio/extend case changes 28 B from offset 0; key-turbo-toggle-cro changes
3 B (120, 121, 126); macro-fl1-cycle changes 3 B (183-185); l2-deadzone-hairtrigger
16 B (1697-1706 and more); r2-deadzone-anti 12 B (1733-1739); left-stick-advance-map
15 B (1762-1775); right-stick-curve 11 B (1809-1815); aim-motion-axis 2 B (1833, 1837);
ext8-touch-sensitivity 5 B (1915, 1916, 1922-1924).

## 14. Unresolved

- Linear curve point values (section 9): two sources disagree.
- Report 18 button byte map beyond the trigger and FL1/FR1/M/Ext bits (section 12).
- Meaning of the set-stick-centre args `n`, `i` (section 11).
- Rounding of the packet count: ceil inferred, the matching index helper floor inferred.
- Encoder return value not confirmed as a byte array.
- Profile read slots 6-7 left unset by the vendor app.
- Notify `t[0]`, `t[1]`.
- Motion block value2 and value3 names (section 7).
- FunData reserved 15 bytes: not read by decode.
- Whether PID 0x090D's profile length differs (section 0).
- Hardware: nothing verified on device. All facts are static or vendor-exec.
