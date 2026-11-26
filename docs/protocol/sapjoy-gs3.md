# SAPJoy "gs3" config frames (T3XS, and T4N2 / N2S on their 0xFF7A interface)

Status: request frames implemented (opensir-core `src/gs3.rs`) and tested byte-for-byte against frames produced by running the vendor library. Replies and hardware behaviour not verified (no device available).

## Source of the vectors
A private harness (not published) loads the vendor `sapjoyuniapis` T4N2 library (v2606151, x86) under unicorn emulation with Windows stubs, runs its `DllMain`, then calls the real exports. Only the device layer is replaced: the HID enumeration returns one fake node (VID 0x3537, usage page 0xFF7A), open/close are no-ops, the raw write is recorded, the reader returns an empty success. Output: `sapjoy-gs3-vectors.json` (selector paths A and B, which the DLL uses for VID 0x3537 and 0x413E, build identical frames) and `sapjoy-t4n2-hid-vectors.json` (the DLL's other branch, see below). Deterministic across runs.

## Frame
`00 A5 L C0 C payload.. S`
- byte 0: report prefix, 0
- `A5`: header; `C0`: request tag (replies use `C1`)
- `L` = payload length + 5
- `C`: command id (table below)
- `S` = sum of bytes 1 .. end of payload, mod 256

Reply (reader rules, `sapjoy-t3xs-n2s.md` section 2.4): `A5 L C1 C status payload..`; the reader accepts it when byte 0 is `A5`, byte 2 is `C1` and byte 3 echoes the command.

## Commands and payloads (all fields u8 unless noted)
| Id | Command | Request payload |
|---|---|---|
| 1 / 2 / 3 / 38 | get device info / MAC / version / device type | none |
| 6 / 7 | set / get D-pad | 4: type (8-way 0, 4-way 1), swap mode (off, D-pad->LS, D-pad->RS, LS<->RS), layout, smart start/stop / none |
| 8 | ABXY rotation test | 1: 0 Xbox, 1 Nintendo |
| 9 / 10 | set / get key mapping | 17: page, key, trigger mode, colour (h u16 LE, s, v), mapping key, toggle, turbo mode, turbo count, mapping type (vacant, joystick, keyboard, mouse, media), gamepad key count, 4-byte code / 1: key |
| 12 | set light | 3: brightness, speed, mode (off, on, breathing, phantom, streamer, radar) |
| 14 | set light colour | 17: light name (home, pedestal, left, right, panel, ABXY, BAYX), 4 x colour (h u16 LE, s, v) |
| 15 | get light colour; the DLL also sends 15 (not 13) to read the light parameters | none |
| 16 | record macro | 2: key, mode (0 start, 1 stop) |
| 17 / 18 | set / get macro step | 16: key, step index, trigger mode, loop (once, cycle), loop time u16 BE (10 ms), mapping type, gamepad key count, 4-byte code, start time u16 BE, end time u16 BE / 2: key, index |
| 19 / 20 | set / get stick curve | 22: stick, page, key, trigger, colour (4), type, ratio, 5 points (x, y), curve type, curvature / 1: stick |
| 21 / 22 | set / get stick mapping | 12: stick, angle, inner range, outer range, stick direction x/y, mode, state, mouse sensitivity x/y, mouse direction x/y / 1: stick |
| 23 / 24 | set / get trigger | 16: trigger, type (linear, adaptive fast, fixed fast), trigger value, release value, 5 points, curve type, curvature / 1: trigger |
| 25 | vibration test | 4: left grip, right grip, left trigger, right trigger |
| 26 / 27 | set / get vibration | 6: left/right grip level, left/right trigger mode (native, trigger sync, grip sync), left/right trigger level / none |
| 28 | gamepad state reports | 1: 0 off, 1 original, 2 mapped |
| 29 / 30 | set / get motion curve | 20: page, key, trigger, colour (4), ratio, 5 points, curve type, curvature / none |
| 31 / 32 | set / get motion mapping | 11: angle, output range, mode (off, LS, RS, key, mouse), type (shooting, racing), shooting mode, key, trigger, stick sensitivity x/y, stick direction x/y / none |
| 35 | save config page | none |
| 45 / 46 | set / get gamepad parameters | 9: sleep minutes u16 LE, auto sleep, 6 reserved / none |
| 47 / 48 | set / get report rate | u16 LE (Hz, e.g. 1000, 8000) / none |

Key ids used in `key` fields: B 0, A 1, Y 2, X 3, L1 4, L2 5, L3 6, R1 7, R2 8, R3 9, minus 10, plus 11, D-pad left/right/up/down 12-15, outer stick directions 24-31, gyro directions 32-35, M1.. 36.., none 255 (inner stick directions 0xEF-0xFE).

## The DLL's other branch (`sapjoy-t4n2-hid-vectors.json`)
When neither 0xFF7A selector finds a device, the library uses older 64-byte frames (`00 F3 01 05 mode cs` for gamepad state, `00 1E ..` ABXY test, `00 F6 ..` macro, `00 25 ..` vibration, `00 23 ..`/`00 F0 ..` stick curve and D-pad, `00 F5 ..` gamepad parameters). In that branch key mapping, trigger, light colour and report rate return "not supported", and stick mapping sends nothing. The captured frames are kept as vectors; OpenSir does not build them.
