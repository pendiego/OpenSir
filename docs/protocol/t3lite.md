# T3Lite (Tegenaria Lite) protocol

Status: observed-static, from the vendor app's behaviour. Golden vectors: `docs/protocol/t3lite-vectors.json`. No hardware was accessed; every field below is `observed-static` unless marked `inferred`.

Labels: `observed-static` (observed from the vendor app), `observed-hw` (none yet), `inferred`.

## 1. Device selection

| Field | Value | Source |
|---|---|---|
| Product ID (wired) | 0x1093 (4243) | observed-static, model table entry |
| Usage page | 65520 | observed-static, same entry |
| Gamepad usage page | -1 | observed-static, same entry |
| Product name / type | "Tegenaria Lite" / "T3Lite" | observed-static |
| Driver | a T3Lite-specific driver class | observed-static |
| Transport | pure HID; no DLL (`HandleByDll` absent) | observed-static |
| Wireless entry | none | observed-static |
| `features` | none | observed-static |

The T3Lite driver has no `bindDevice` step, so it does not use the C2 attach path (`cyclone2-transport.md` section 5).

## 2. Outbound writes

The T3Lite driver registers its own handlers and also receives the app's generic command dispatch. Every write goes to the device's HID handle with the bytes as given. The vendor app does not pad the buffer to a fixed length here. Whether the HID layer pads it: not verified.

Command bytes come from the vendor app's shared command builders.

| Command | Bytes (hex) | Sent by | Notes |
|---|---|---|---|
| Vibrate | `0f 20 66 55 L R` | T3Lite driver, on a vibrate request | Zeroed after `duration \|\| 1000` ms. Motor range not verified. |
| Quick update | `0f 17 55 88` | T3Lite driver, on a quick-update request | Sets the manual-updating flag first. Device response: not captured. |
| Read RGB | `0f 0d` | generic dispatch, on a get-RGB-state request | Not handled by the T3Lite driver itself. |
| Set light status | `0f 0d on frame` | generic dispatch, on a set-light-status request | Same opcode (0x0d) as Read RGB. Meaning of the second byte: inferred. |
| Set macro status | `0f 16 r t` | generic dispatch, on a set-macro-record-status request | `r` = 1 for FL1, 2 for FR1. Used only with FL1/FR1 on this model: inferred. |

Builders the T3Lite driver never calls. Listed for completeness; documented from the builder only:

| Command | Bytes | Notes |
|---|---|---|
| Enter profile config | `0f 01` | not sent by the T3Lite driver |
| Exit profile config | `0f 02 save` | not sent by the T3Lite driver |
| Switch profile | `0f 07 n` | reaches the T3Lite driver through the generic dispatch; profile index 5 = Shift (`ShiftProfileNumber`): inferred for this model |
| Read FW version | `0f 09` | not sent by the T3Lite driver |
| Read current profile | `0f 0b` | not sent by the T3Lite driver |
| Set calibration state | `0f fd on` | not sent by the T3Lite driver |
| Heartbeat | `0f f2 00` | the T3Lite driver's heartbeat is empty. Do not treat as sent. |

Not documented as protocol: the upgrade-mode command `0f fc`. It is reached only through the DFU path. Do not send it.

## 3. Inbound reports

### 3.1 Gate

The driver's report handler (`receivedGamepadData`) acts only when `t[0] == 16` and `t[1] == 20`. So the report id is 0x10 and the second byte is the length tag 0x14. Other reports are dropped. If the report differs from the previous one, it is stored and passed to the input decoder, then to `parseGamepadState`.

The C2 input (`cyclone2-input.md`, `GamepadKeyStateReportId` 18) uses a different layout. Do not reuse C2 offsets for T3Lite.

### 3.2 Byte map

Indices are into the report buffer `t`, with `t[0] = 0x10`.

| Index | Field | Decode |
|---|---|---|
| 2 bits 4-7 | Share (4), Options (5), L3 (6), R3 (7) | bit test |
| 2 low nibble | Directional | hex digit (see 3.3) |
| 3 bit 0 | L1 | bit test |
| 3 bit 1 | R1 | bit test |
| 3 bit 2 | Home | bit test |
| 3 bit 4 | CRO | bit test |
| 3 bit 5 | CIR | bit test |
| 3 bit 6 | SQU | bit test |
| 3 bit 7 | TRI | bit test |
| 4 | LeftTrigger (raw); L2 = (value == 255) | raw byte |
| 5 | RightTrigger (raw); R2 = (value == 255) | raw byte |
| 6-7 | left stick x, signed 16-bit LE | scaled, 3.4 |
| 8-9 | left stick y, signed 16-bit LE | scaled and negated, 3.4 |
| 10-11 | right stick x | scaled, 3.4 |
| 12-13 | right stick y | scaled and negated, 3.4 |
| 14 bit 0 | M | bit test |
| 14 bit 1 | Camera | bit test |
| 14 bit 2 | FL1 | bit test |
| 14 bit 3 | FR1 | bit test |
| 14 bit 4 | turbo on | bit test (in `parseGamepadState`) |
| 15 | colour R | byte |
| 16 | colour G | byte |
| 17 | colour B | byte |
| 18 | l3Raw = (value == 1) | byte |
| 19 | r3Raw = (value == 1) | byte |
| 20 bit 0 | RealB | bit test |

Bits in index 2 and 3 are read as `(e[i] & (1 << b)) != 0`.

### 3.3 Directional (D-pad)

The D-pad is the lowest hex digit of byte 2, `n`. Flags as the vendor app sets them:

| Flag | Set when `n` is |
|---|---|
| DPad_Up | 1, 5, 9 |
| DPad_Up_Right | 9 |
| DPad_Right | 8, 9, a |
| DPad_Right_Down | a |
| DPad_Down | 2, 6, a |
| DPad_Down_Left | 6 |
| DPad_Left | 4, 5, 6 |
| DPad_Left_Up | 5 |

The flag sets overlap: one digit can set several flags. For example `9` sets Up, Up-Right and Right. Digits 0, 3, 7, b-f set no D-pad flag. The idle value is `"0"`. Whether the device reports these digits this way is not verified.

### 3.4 Stick scaling

Each axis is a signed 16-bit little-endian value `v`. The vendor app scales it as follows:

1. `t = v - 128`. The offset is applied to the signed value, not to a byte.
2. `t == 0` gives 0.
3. `t > 0` gives `round(1e4 * t / 32767) / 1e4`.
4. `t < 0` gives `round(1e4 * t / 32768) / 1e4`, where `round` is JavaScript `Math.round`.

Y axes are negated after scaling. Consequences, from the vectors:

- Raw `0x0000` gives x = -0.0039, not 0. A raw `0x0080` (128) gives 0.
- Raw `0x8000` (-32768) gives -1.0039 after the negation on y. The result can go past ±1.

Whether the physical neutral position is `0x0080` or `0x0000`: `inferred`, not measured.

Stick direction flags use the scaled values: `LS_Up` when `-0.5 > y`, `LS_Down` when `0.5 < y`, and the same for x and the right stick.

### 3.5 Change event

The decoder emits `gamepadStateChanged` with arguments `(null, path, prev, next)` when the state differs from the previous one. The first report after start emits nothing.

## 4. Parse side effects in the T3Lite driver

`parseGamepadState`:

- On a change in `{l3Raw, r3Raw, turboOn}`, dispatches `updateDeviceProperties` with `{deviceId, state}`.
- Emits a get-RGB-state ack with `{rgb: {R, G, B}, deviceId}` on a throttle. `syncRgbCount` is read before increment: while the pre-increment value is below 5, nothing is emitted; at 5 the emit fires and the counter resets to 0. Vectors: emit on the 6th and 12th parses.

## 5. Vectors

`docs/protocol/t3lite-vectors.json`:

- `commands`: 17 entries with `bytes_hex`, from the command builders and generic dispatch. `ds_sends` says whether the T3Lite driver itself sends it.
- `input_reports`: synthetic 21-byte reports, byte 0 = 0x10, byte 1 = 0x14.
- `decoded_states`: 7 named reports run through the input decoder: `idle_zero`, `sticks_centred_0x80`, `sticks_full_left_down`, `buttons_overlap_byte2`, `triggers_full`, `turbo_colour_flags`, `byte14_bits_no_turbo`.
- `ds_parse`: `updateDeviceProperties` dispatch and the RGB-ack emit positions.
- `receive_gate`: a report with byte 0 = 0x12 gives no decoder event and no parse. A 16/20 report gives one event and one parse.

Inputs are synthetic. Outputs come from running the vendor app's logic. Nothing was checked against a real device.

## 6. Open questions

- Opcode `0x0d` is shared by Read RGB and Set light status. The T3Lite caller of each is not confirmed: the T3Lite driver does not handle either one. Which one the device answers is unknown.
- The vibration motor range (is `L` and `R` 0-255?) is not checked.
- Whether the HID layer pads writes to a fixed report length (the vendor app does not).
- Device response to quick update `0f 17 55 88`, and to any command. No responses were captured.
- D-pad digit mapping: flags overlap and digits 0, 3, 7, b-f set none. Confirm on hardware.
- Neutral stick value (0x0080 or 0x0000).
- Meaning of the second byte of Set light status (`frame`) and of the macro status values `r` and `t`.
- Whether the profile builders (`0f 01`, `0f 02`, `0f 07`, `0f 09`, `0f 0b`, `0f 16`) work on T3Lite. Only the code's own sends are documented.
