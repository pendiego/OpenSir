# Cyclone 2 config-channel transport

Scope: device selection, framing, command send/ack semantics, attach sequence, inbound packet classification, input reports. Profile binary layout and UI are out of scope.

Labels: `observed-static` (observed from the vendor app's behaviour), `observed-hw` (read-only probe on a real device), `inferred`.

## 1. Device selection

| Field | Value | Source |
|---|---|---|
| Vendor ID | 0x3537 | observed-static, Modes ConfigModeVid |
| Product ID (wired) | 0x101D (4125) | observed-static, vendor device table |
| Product ID (wireless) | 0x102A (4138) | observed-static |
| Product ID (wired ADC) | 0x1053 (4179) | observed-static |
| Product ID (wireless ADC) | 0x100B (4107) | observed-static |
| Config usage page | 0xFFF0 | observed-static (UsagePage 65520) |
| Config usage | 0x40 | observed-hw (HID collection) |
| Hardware seen | PID 0x0575, rev 0x0121, "GameSir-Cyclone 2" | observed-hw |

Open question: the connected unit reports PID 0x0575, which is not in the C2 `Modes` table. How the vendor accepts it (another device-table entry, or a `HandleByDll` path) is unresolved.

Matching rule (vendor supported-device check): vid and pid must equal a `ConfigModeVid`/`ConfigModePid` pair, and usage page must match when both sides set it.

## 2. Framing

- Every outbound command is an array whose first byte is `15` (0x0F). Byte 1 is the command id. The rest are arguments.
- The builder does not pad to 64 bytes, except `writeProfile` (`Array(64)`).
- On this Windows host, `hid.write` of a raw array returns 64 (hidapi pads). Writes with a leading 0x00 report id return -1.

## 3. Command table (vendor command builders, observed-static)

Golden vectors: `docs/protocol/cyclone2-command-vectors.json`, vendor encoder output observed via a local harness.

| Command | Bytes (args) | Class |
|---|---|---|
| EnterProfileConfig | `[15,1]` | WRITE |
| ExitProfileConfig(f) | `[15,2,f?1:0]` | WRITE |
| ReadProfile(n) | chunks `[15,4,n,off_hi,off_lo,len]`, 58-byte chunks; total 680 (635 for n=32) | READ |
| ReadAudio(n) | `[15,4,n,0,40,5]` | READ |
| SwitchProfile(n) | `[15,7,n]` | WRITE |
| ReadFWVersion | `[15,9]` | READ |
| ReadCurrentProfile | `[15,11]` | READ |
| EnterUpgradeMode | `[15,252]` | DFU, never send |
| HeartBeat(e) | `[15,242,e]` | read-only, inferred |
| ReadRGB | `[15,13]` | READ |
| Vibration(e,t) | `[15,32,102,85,e,t]` | WRITE |
| SetLightStatus(e,t) | `[15,13,e?1:0,t]` | WRITE (same id as ReadRGB) |
| ReadDeviceInfo(e) | `[15,4,e,...,3,157,7]` | READ, exact bytes unconfirmed |
| SetDeviceId(e,t) | `[15,3,e,3,158,6]` + ascii | WRITE |
| WriteProfile(e,t,r) | `[15,3,t,off_hi,off_lo,len,data]` | WRITE |
| RefreshProfileByOffset | `[15,16,e,off_hi,off_lo,len]` | classify before use |
| SetCalibrationState(e) | `[15,253,e]` | WRITE |
| SetMacroStatus(e,t) | `[15,22,r,t]` | WRITE |
| QuickUpdate | `[15,23,85,136]` | never send |

## 4. Send queue and ack (observed-static)

- `writeWithAck(deviceId, bytes)` queues; `write()` queues without ack; `writeImmediately(path, bytes)` bypasses the queue.
- Queue pump runs every 20 ms. Status 0 → send. Ack-required → status 1, else status 2.
- Status 1 resends after 200 ms, up to about 10 sends, then status 3 (cancelled). Status 3 after 2 failures triggers `loadProfileFailed` and clears the queue.
- Ack rule: any recognised inbound packet on the device path pops the queue head. Replies are not matched to the command by id.

## 5. Attach sequence (observed-static)

1. Open HID config interface; immediately write HeartBeat `[15,242,0]`.
2. Register `recvdata` handler.
3. After 100 ms: read profiles 1-4, then Shift (5), then Light (32) via chunked reads.
4. Write ReadFWVersion and ReadCurrentProfile via `writeWithAck`.
5. Vendor sends HeartBeat every 1 s while focused.

## 6. Inbound classification (observed-static)

Class is taken from byte index 1.

| Byte 1 | Class |
|---|---|
| 1 / 2 | EnterProfileConfig / ExitProfileConfig |
| 3 | WriteProfile |
| 4 / 5 | ReadProfile / ReadProfileAck |
| 6 | Ack |
| 7 | SwitchProfile |
| 8 | WriteProfileToEEPRom |
| 9 / 10 | ReadFirmwareVersion / Ack |
| 11 / 12 | ReadCurrentProfile / Ack |
| 13 / 14 | ReadRGB / Ack |
| 15 / 16 / 17 | ProfileChanged / RefreshProfile / Ack |
| 18 / 19 | WriteEEPRom / Ack |
| 20 / 21 | ReadEEPRom / Ack |
| 240 / 241 | Download / Ack |
| 242 | HeartBeat |
| 243 / 244 | ReadKeyStatus / Ack |
| 252 | RequestToUpgrade |
| other | Unknow |

Refinements: ReadAudioAck when bytes 6,7,8 = 0,40,5. ReadProfileComplete when class is ReadProfileAck and `256*b3 + b4 + b5` equals 635 (profile 32) or 680 (others). AckWithBusy when class is Ack and byte 2 is 1.

## 7. Input reports

- Report id 18 (0x12) is the gamepad state, not the config channel. Parsing (observed-static): byte 53 bits 0-1 macro record state; bit 4 → FL1 record; bit 5 → FR1 record. Bytes 35-36 charge state and battery level; then config index and five RGB triples.

## 8. observed-hw results (read-only probe on a real device)

- Enumeration: 3 HID interfaces for 3537:0575: 0xFF32/0x74 (Col02), 0xFFF0/0x40 (Col03, config), 0x000C/0x01 (Col01, consumer).
- Writes to 0xFFF0 are accepted (see section 2).
- Sent, no reply within 0.3-1.0 s each, on both 0xFFF0 and 0xFF32: heartbeat `[15,242,0]`, ReadFWVersion `[15,9]`, ReadCurrentProfile `[15,11]`, ReadRGB `[15,13]`, ReadProfile chunks for slots 1-5 and 32 (one retry each). Result: 0 of 71 chunks acknowledged.
- Passive 5 s listen on 0xFFF0: 5001 input reports, all report id 18 (0x12), idle payload (zero bytes). No other packet classes observed.
- Conclusion: this unit does not answer the config read commands in the vendor's form. Unverified causes: the vendor session needs state from a write we are not allowed to send (e.g. EnterProfileConfig `[15,1]`), or the framing differs. Needs user approval before further probing.
- Superseded by the vendor log below: the 0575 result is explained (non-config mode). The probe was a dead end for PID 0x0575.

### 8a. Vendor-log findings (observed-hw, the vendor app's own log captured on a real device)

- PID 0x0575 is logged as `device added: ...(3537/0575) ... nativeSupport: false` (seen three times; verified). It is a non-config mode. The vendor drives the controller on the 0x100B path (`PID_100B&MI_01&Col04`, verified in the log). The 0x1053 path is per Main only (not seen in the log). Do not send commands to 0575.
- Config interface is `MI_01&Col04` on the 0x1053/0x100B paths (per Main; not re-verified here).
- Log format caveat (`observed-static`): `[Recv]` lines are made by mapping each byte through a 2-digit hex formatter over a Node `Buffer`, so the hex text is coerced back into a byte. A byte whose hex has a letter (0x0A, 0x2E, 0x4B...) prints as `0`; a byte like 0x10 prints as `10`. Read `[Recv]` lines that way. The separate `ReadProfileComplete <hex>` lines are plain hex and exact.
- Framing (settled): outbound report id 0x0F at byte 0. Inbound config replies print `10` at byte 0, so they arrive on report id 0x10; byte 1 is the class (Ack prints `6`; FW ack prints `0` because 0x0A has a letter). The vendor treats every report whose byte 0 is not 0x12 as a config reply. Gamepad state is report 0x12.
- Ack: not verified in the log. The `[Recv] Ack` and WriteProfile timing lines I cited earlier could not be confirmed on disk; treat ack latency as unmeasured.
- Resend: not verified in the log; chunk length 58 is the probe's value only (section 3).
- ReadFirmwareVersionAck, decoded with the caveat above: byte 1 = 0x0A, bytes 2.. are ASCII with separators that print as `0` (letters in hex, consistent with `.` = 0x2E). The vendor reads the firmware as UTF-8 of bytes 4..9 and the dongle version as bytes 12..17. For this unit that gives firmware `3.4.0`-like text and dongle `1.2.1`, which matches the bundled packages `GS_C2_ADC_DEVICE_V352` (newer) and `GS_C2_Dongle_V121` (same). `inferred` for the separator byte.
- ReadCurrentProfileAck `10,0,1`: byte 1 = 0x0C (prints `0`), byte 2 = current profile 1.
- ReadProfileComplete `10,5,<slot>,2,0,0,...`: slot at index 2, class 5 (verified for slots 1, 2, 3). Lines are about 0.45 s apart (21:02:48.997, 49.439, 49.904).
- Write lines (`WriteProfile(Index: Light, Offset: 0x175, Length: 122)`, `Offset: 0x17a, Length: 13`) are vendor evidence only; this worker does not send them.
- Connection (verified in the log): `conenction result: false` (21:04:28.801), `retry to connect devices, count: 1` (.952), `conenction result: true` (.982). About 30 ms from retry to success.

### 8b. Mode identification (observed-static)

- The vendor's supported-device check `isSupportedDevice(vid,pid,usagePage)` is true only when a `Modes` entry matches vid and pid, and usage page is 65535 (any) or equal. The vendor's device filter keeps only supported devices or supported gamepads. The log's `nativeSupport:` is this same value.
- C2 Modes: wired 4125 (0x101D), wireless 4138 (0x102A), wired ADC 4179 (0x1053), wireless ADC 4107 (0x100B). These are the only config-capable PIDs.
- No unsupported-mode prompt was found in the vendor UI (searched for mode/XInput/DInput text). The app silently ignores non-native devices. The only related text is the 1000 Hz `M + HOME` hint.
- Config needs a PID from the Modes list. The code does not require a specific XInput/PC mode; the enumerated PID decides.


## 9. Open questions

1. PID 0x0575 vs. the C2 Modes table (section 1).
2. Report-id framing on the wire; whether unpadded arrays are accepted.
3. Heartbeat `[15,242,0]` read-only status: vendor sends it, but this is inferred.
4. ReadDeviceInfo exact bytes.
5. Classifier tail beyond 244.
6. Why no config reply on this unit (section 8).
