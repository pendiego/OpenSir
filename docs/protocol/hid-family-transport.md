# HID family transport, input and commands: T3CE, G7ProCE, K2_8K

Diff against Cyclone 2. Baseline: `cyclone2-transport.md` (§3 commands, §4 send queue, §5 attach, §6 inbound classes, §7 input report), `cyclone2-input.md`, `cyclone2-sequencing.md`.

Source: observed from the vendor app's behaviour (`observed-static`). Byte values are facts; no vendor code is reproduced. Profile layouts are in `t3ce.md`, `g7proce.md`, `k2-8k.md` and are not repeated here.

"Same as C2" means the same bytes as C2 (`cyclone2-transport.md` §3). "Same as T3CE" means the same bytes as the T3CE command.

## 1. Identity and interface

| Model | Mode | PID (dec / hex) | Notes |
|---|---|---|---|
| T3CE | Wired | 4157 / 0x103D | config VID 13623 (0x3537), UsagePage 65520 (0xFFF0) |
| T3CE | Wired (variant) | 2317 / 0x090D | KeyboardAndMouse, MapToMoreKeys, Turbo disabled; hides /motion |
| G7ProCE | Wired | 4279 / 0x10B7 | |
| G7ProCE | Wireless | 4281 / 0x10B9 | has a packed wireless firmware (`PackedFirmware`) |
| G7ProCE | Wired (variant) | 4301 / 0x10CD | has its own features block |
| G7ProCE | other PIDs | 4302 / 0x10CE, 4293 / 0x10C5, 4294 / 0x10C6, 4349 / 0x10FD, 4350 / 0x10FE, 4295-4300 / 0x10C7-0x10CC, 4146-4149 / 0x1032-0x1035, 4347 / 0x10FB, 4348 / 0x10FC | Mode label not read. |
| K2_8K | Wired | 4238 / 0x108E | |
| K2_8K | Wireless | 4255 / 0x109F | has a packed wireless firmware (`PackedFirmware`) |

Usage page 65520 is used by all three. Each model lists its wired/wireless modes; the app selects a device by matching these PIDs. Interface choice (`MI_01&Col04` for 0x1053 / 0x100B on C2) is not verified for these PIDs.

C2 reference PIDs: wired 4125 (0x101D), wireless 4138 (0x102A), ADC wired 4179 (0x1053), ADC wireless 4107 (0x100B).

PID 0x0575 is a non-config mode (seen in the vendor app's log). Do not send config commands to it.

## 2. Framing

Same as C2 (`cyclone2-sequencing.md`): report id 0x0F at byte 0, 64-byte output, 58-byte maximum payload. Inbound config replies use report id 0x10. Gamepad state uses report id 18 (0x12). All three use `EveryOutPacketLength` 58.

Send queue, attach, heartbeat cadence and cancel rules are the C2 ones (`cyclone2-transport.md` §4, §5).

## 3. Profile and report sizes

| Constant | T3CE | G7ProCE | K2_8K | C2 |
|---|---|---|---|---|
| EveryPacketLength (full profile) | 1935 | 1070 | 1408 | 680 (slot 32: 635) |
| LightProfilePacketLength | 102 | 165 | 229 | |
| EveryPacketStartIndex | 5 | 5 | 5 | |
| GamepadKeyStateReportId | 18 | 18 | 18 | |
| ShiftProfileNumber | 5 | 5 | 5 | |
| LightProfileNumber | 32 | 32 | 32 | |
| CurrentOfProfileNumber | 48 | 48 | 48 | |
| MacroStepMaximum | 32 | 32 | 32 | |
| FunctionKeyPacketLength | 169 | 169 | 169 | |
| StickPacketLength | 36 | 36 | 36 | |
| TriggerPacketLength | 32 | 32 | 32 | |
| SensorPacketLength | 41 | 41 | 41 | |
| FirmwareVersionPacketLength | 8 | 8 | 8 | |
| SingleAnimationPacketLength | n/a | 32 | 44 | |
| AnimationCount / AnimationFrameCount | n/a | 5 / 8 | not read | |
| RGBCount | n/a | 5 | not read | |

Stick constants are the same in all three (`Stick` min/default/max 1800/2400/3000 and trigger 180/540/1080).

## 4. Outbound commands

All command bodies below are `[15, ...]` unless noted. Bytes are as the app writes them; `slot`, `off`, `len`, `e`, `t` are parameters.

| Command | Bytes | C2 | T3CE | G7ProCE | K2_8K |
|---|---|---|---|---|---|
| Read profile chunk | `[15,4,slot,off_hi,off_lo,len]`, chunks of 58 | §3 | yes (1935; slot 32 uses 102) | yes | builder not separately read |
| Switch profile | `[15,7,slot]` ("Shift" maps to 5) | same | yes | yes | yes |
| Read FW version | `[15,9]` | same | yes | yes | yes |
| Read current profile | `[15,11]` | same | yes | yes | yes |
| Heartbeat | `[15,242,e]` | same | yes | yes | yes |
| Read RGB | `[15,13]` | same | yes | yes | yes |
| Vibration | `[15,32,102,85,e,t]` | same | yes | yes | yes |
| Set light status | `[15,13,e?1:0,t]` | same | yes | yes | yes |
| Refresh by offset | `[15,16,slot,off_hi,off_lo,len]`, chunks of 58 | §3 | yes | yes | yes |
| Write profile | `[15,3,slot,off_hi,off_lo,len,data]`, chunks of 58 | §3 | yes | not separately read | not separately read |
| Quick update | `[15,23,85,136]` | never send | yes (sent behind a user prompt) | yes | yes |
| Set macro status | `[15,22,key,t]` | same | used (builder not read) | yes | yes (key is the key's map value) |
| Stick centre | `[15,226,side,dir,off_hi,off_lo,...]`, 64-byte | none | yes; builder default offset 16, the app sends offset 32 | yes; the app sends offset 32 | none (no builder, no app method) |
| Calibration state | `[15,253,e]` | C2 only | not seen | not seen | not seen |
| Enter / exit profile config | `[15,1]` / `[15,2,f]` | same | not seen | not seen | not seen |

Stick centre payload: side Left = 0, Right = 1; direction left = 0, right = 1, up = 2, down = 3; offset 32. Invalid side or direction logs a warning in the G7 app. The app checks `undefined` for both values.

Stick centre ack on T3CE: `SetStickCenterAck`, byte `t[2]` = 0 means success. G7 ack handling is not read.

Profile write chunks: T3CE 58-byte payload per chunk; the app computes a byte diff before writing. G7 and K2 write path was not read.

## 5. Inbound dispatch

Common across all three: the report is checked against byte 0. Report 18 is gamepad state. Any other report is a config reply, and its class is decided by the same reply-class rules as the C2.

Class rules that are the same as C2 (`cyclone2-transport.md` §6): AckWithBusy (byte 2 == 1), ReadProfileAck, ReadProfileComplete, ReadFirmwareVersionAck, ProfileChanged, UsbModeChanged. ReadAudioAck is recognised when bytes 6/7/8 are 0/40/5 (T3CE).

ReadProfileComplete is decided by `256*b3 + b4 + b5`. Value 32 means light profile (102 bytes). Otherwise 1935 (T3CE). G7 and K2 decode it with their own profile and light codecs.

| Class | T3CE | G7ProCE | K2_8K |
|---|---|---|---|
| Gamepad (report 18) | skipped; routed to the gamepad parser | skipped; routed to the gamepad parser | skipped; routed to the gamepad parser |
| CalibrationResponse | `t[5]` mode, `t[6]` bits: LT 0, RT 1, LS 2, RS 3, Sensor 4 | not read | not read (K2 reports `CalibrationStateChanged` from the gamepad report) |
| SetStickCenterAck | yes, `t[2]` = 0 is success | not read (G7 app has the method; ack branch unverified) | no stick-centre path |
| ReadRGBAck | not a branch; RGB arrives in the gamepad report | not a branch | yes (§7) |
| MacroStatusAck | not a branch | not a branch | yes (§7) |
| ProfileChanged | `t[2]` profile, offset `256*t[3]+t[4]`, length `256*t[5]+t[6]`; refresh by offset; profile 48 triggers read current profile | same refresh; profile 48 triggers read current profile | same refresh; then re-reads current profile |
| ReadFirmwareVersionAck | resolves firmware version | resolves firmware version; if wireless, sets `ControllerFirmware.hasUpdate` from the packed firmware's update check and emits `deviceLoaded` | same, with the same wireless DFU check |
| ReadCurrentProfileAck | forces profile 1 if >4 or Shift | same rule | same rule |

Shift profile = 5, light = 32.

## 6. Gamepad report (report 18)

Parsers: T3CE uses a three-argument input decoder. G7 and K2 share a decoder with an optional fourth argument `n`: when set, it passes `t.slice(n, n+4)` into the second decode of bytes 53-63. G7 calls it with `26`. K2 does not pass it.

The decoder internals were not read, so the meaning of those four bytes is not established.

Status block, per model (raw index into the 64-byte report):

| Field | T3CE (slice 25..51) | G7ProCE (slice 35..55) | K2_8K (slice 35..55) |
|---|---|---|---|
| calMode | 25 | 31 | 31 |
| calStatus | 26 | 32 | 32 |
| chargeState | 27 | 35 | 35 |
| batteryLevel | 28 | 36 | 36 |
| macro record key | 29 | 53 (non-zero only, mapped to a button key) | none in report; looked up by index with MacroStatus |
| ConfigIndex | 30 | 37 | none |
| RGB triples | 31-48 (6 × 3: Panel1-5 and Home) | 38-52 (5 × 3: Home, LowerLeft, LowerRight, UpperLeft, UpperRight) | none |
| Extended buttons | bytes 53-63 | bytes 53-63 (decoded with the fourth argument) | bytes 53-63 |

calStatus bits (T3CE 26 and K2 32, same map): bit 0 LS, 1 RS, 2 LT, 3 RT, 4 Sensor. Note that the CalibrationResponse byte `t[6]` on T3CE uses a different order (LT 0, RT 1, LS 2, RS 3, Sensor 4). G7 calStatus bit map was not read; it is assumed to match T3CE/K2.

G7 also throttles RGB-count sync. K2 forwards `CalibrationStateChanged` as a device event.

## 7. Model-specific inbound replies

### K2_8K
- `ReadRGBAck`: needs `length >= 33` (3 + 3 × 10). Byte 2 = ConfigIndex. Ten 3-byte groups (R, G, B) from byte 3 to byte 32, zone ids [5,4,3,6,7,8,9,10,1,2] with names Home, UpperLeft, UpperRight, ProfileSideLeft, ProfileMiddleLeft, ProfileMiddle, ProfileMiddleRight, ProfileSideRight, LowerLeft, LowerRight. Emits `reqGetRgbStateAck` with `deviceId`.
- `MacroStatusAck`: needs `length >= 7`. Byte 2 = macro key index, byte 4 = status. Status 1 or 3 sets recording state with the key. Status 4 clears recording state (state 2, key 0). Other values warn.

### G7ProCE
- No `ReadRGBAck` or `MacroStatusAck` branches. RGB and macro state arrive through the gamepad report (bytes 37-53).

### T3CE
- No dedicated RGB or macro ack branches. Config state arrives through the gamepad report (bytes 29-30).

## 8. Diff summary

| Item | T3CE | G7ProCE | K2_8K |
|---|---|---|---|
| Builders (read, switch, FW, current, heartbeat, RGB read, vibration, light, refresh, macro, quick update) | same bytes as C2 and T3CE | same as T3CE | same as T3CE |
| Profile read total / light | 1935 / 102 | 1070 / 165 | 1408 / 229 |
| Stick centre | yes, offset 32 | yes, offset 32 | no |
| Input push fourth arg | none | `26` | none |
| Gamepad calMode/calStatus | 25/26 | 31/32 | 31/32 |
| Battery/charge | 28/27 | 36/35 | 36/35 |
| Macro record key in report | byte 29 | byte 53 | not in report |
| RGB in report | 6 triples at 31-48 | 5 triples at 38-52 | none |
| RGB reply | none | none | `ReadRGBAck` 10 zones |
| Macro status reply | none | none | `MacroStatusAck` |
| ProfileChanged refresh | refresh by offset | refresh by offset | refresh by offset |
| Wireless DFU hasUpdate | no | yes | yes |

## 9. Reuse table for OpenSir's C2 transport

Legend: **reuse** = same code path works unchanged. **param** = needs per-model value. **new** = needs code that C2 does not have.

| Area | T3CE | G7ProCE | K2_8K |
|---|---|---|---|
| 64-byte framing, 0x0F report id, 58-byte chunks | reuse | reuse | reuse |
| Send queue, attach, heartbeat | reuse | reuse | reuse |
| Read profile chunks | param (1935 / slot 32: 102) | param (1070 / 165) | param (1408 / 229); builder read but name not confirmed |
| Switch profile, read FW, read current profile | reuse | reuse | reuse |
| Heartbeat, vibration, set light status, read RGB | reuse | reuse | reuse |
| Refresh by offset | reuse | reuse | reuse |
| Write profile (diff, 58-byte chunks) | reuse (T3CE builder) | verify | verify |
| Quick update | do not send (C2 rule) | do not send | do not send |
| Set macro status | reuse | reuse | reuse |
| Stick centre (64-byte, offset 32) | new (C2 has none) | new (same builder) | not applicable |
| Calibration state `[15,253,e]` | verify (not in builder block) | verify | verify |
| Gamepad status parse (calMode, charge, battery offsets) | param (25/26/27/28) | param (31/32/35/36) | param (31/32/35/36) |
| Gamepad macro record key | new (byte 29) | new (byte 53, non-zero only) | new (via macro status ack) |
| Gamepad RGB (triples) | new (6 zones, 31-48) | new (5 zones, 38-52) | new (ack, 10 zones, 33 bytes) |
| Macro status ack (7+ bytes) | not applicable | not applicable | new |
| RGB ack (33+ bytes) | not applicable | not applicable | new |
| ProfileChanged refresh | reuse | reuse | reuse |
| Wireless DFU hasUpdate | not applicable | new (wireless only) | new (wireless only) |
| Input push fourth arg (G7 26) | not applicable | new (meaning unverified) | not applicable |
| Config interface / PID selection | param (PIDs above) | param (PIDs above) | param (PIDs above) |

## 10. Unverified (do not rely on without re-check)

- G7 calStatus bit map (assumed same as T3CE/K2).
- G7 input decoder fourth argument `26`: the decode of bytes 26-29 passed into the button decoder is established; its meaning is not.
- Command bodies for T3CE set-macro-status, K2 read-chunk, K2/G7 write-profile, and calibration state `[15,253,e]` on T3CE/G7/K2.
- Mode label (wired/wireless) for G7 PIDs other than 4279, 4281, 4301.
- Config interface (`MI_01&Col04` or other) for these PIDs.
- Enter and exit profile config `[15,1]` and `[15,2,f]` on these three models.
- G7 and K2 wireless DFU update-check behaviour beyond what is recorded here.
