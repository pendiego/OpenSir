# SAPJoy T3Pro configuration exports (sapjoyuniapis_x86_v1.0.16.dll)

Status: partial. The export-to-slot map and the macro-step request/reply layouts are recovered. The other config commands are mapped to slots but not decoded. Transport (WriteFile, response read) is in `sapjoy-t3pro.md`.

Labels: `observed-static` (observed in the vendor library's code, address cited), `derived-static` (follows from observed code), `inferred` (not confirmed).

## Export wrappers (observed-static)
Each config export is a thin wrapper. It checks the handle and length, casts the device to `ISAPJoyHID` (cast helper at 0x1004d7d8), then makes an indirect virtual call at `*vtable + off`, forwarding its arguments. Ordinal and RVA come from the library's export table.

| Export | Ordinal | Wrapper RVA | Vtable offset (slot) | Args forwarded |
|---|---|---|---|---|
| SetMacroStep | 37 | 0x2c50 | +0x04 (1) | 1 |
| GetMacroStep | 14 | 0x2ca0 | +0x08 (2) | 3 |
| SetStickCurveHSV | 42 | 0x2ee0 | +0x0c (3) | 1 |
| GetStickCurveHSV | 17 | 0x2f30 | +0x10 (4) | 3 |
| SetStickMapping | 43 | 0x2f80 | +0x14 (5) | 1 |
| GetStickMapping | 18 | 0x2fd0 | +0x18 (6) | 2 |
| SetTriggerParam | 45 | 0x3020 | +0x1c (7) | 1 |
| GetTriggerParam | 20 | 0x3070 | +0x20 (8) | 2 |
| SetMotionSenseCurveHSV | 38 | 0x30c0 | +0x24 (9) | 1 |
| GetMotionSenseCurveHSV | 15 | 0x3110 | +0x28 (10) | 2 |
| SetMotionSenseMapping | 39 | 0x3160 | +0x2c (11) | 1 |
| GetMotionSenseMapping | 16 | 0x31b0 | +0x30 (12) | 1 |
| SetKeyMappingHSV | 33 | 0x3200 | +0x34 (13) | 1 |
| GetKeyMappingHSV | 10 | 0x3250 | +0x38 (14) | 3 |
| SetVibrateLevel | 46 | 0x32a0 | +0x3c (15) | 1 |
| GetVibrateLevel | 22 | 0x32f0 | +0x40 (16) | 1 |
| SetLightParam | 36 | 0x3340 | +0x44 (17) | 1 |
| GetLightParam | 12 | 0x3390 | +0x48 (18) | 1 |
| SetLightColorHSV | 34 | 0x33e0 | +0x4c (19) | 1 |
| GetLightColorHSV | 11 | 0x3430 | +0x50 (20) | 2 |
| SaveConfigPage | 25 | 0x3510 | +0x5c (23) | 0 |
| SetDPadParam | 28 | 0x3590 | +0x64 (25) | 1 |
| GetDPadParam | 4 | 0x35e0 | +0x68 (26) | 1 |

Sources: the export wrappers and the vtable slots. The arg counts come from the wrapper signatures.

## Vtable implementations (observed-static dump, identity inferred)
The vtable is read at VA 0x1005d818 (`.rdata`, file offset 0x5be18, derived-static from the section map). Slot targets for the config slots:

| Slot offset | Target | Export |
|---|---|---|
| +0x04 | 0x10005830 | SetMacroStep |
| +0x08 | 0x100059a0 | GetMacroStep |
| +0x0c | 0x10005b50 | SetStickCurveHSV |
| +0x10 | 0x10005d80 | GetStickCurveHSV |
| +0x14 | 0x10005ff0 | SetStickMapping |
| +0x18 | 0x10006160 | GetStickMapping |
| +0x1c | 0x100062e0 | SetTriggerParam |
| +0x20 | 0x10006450 | GetTriggerParam |
| +0x24 | 0x100065d0 | SetMotionSenseCurveHSV |
| +0x28 | 0x100067f0 | GetMotionSenseCurveHSV |
| +0x2c | 0x10006a20 | SetMotionSenseMapping |
| +0x30 | 0x10006b90 | GetMotionSenseMapping |
| +0x34 | 0x10006cf0 | SetKeyMappingHSV |
| +0x38 | 0x10006ee0 | GetKeyMappingHSV |
| +0x3c | 0x10007120 | SetVibrateLevel |
| +0x40 | 0x10007350 | GetVibrateLevel |
| +0x44 | 0x100074b0 | SetLightParam |
| +0x48 | 0x10007680 | GetLightParam |
| +0x4c | 0x100077e0 | SetLightColorHSV |
| +0x50 | 0x10007a40 | GetLightColorHSV |
| +0x5c | 0x10007f00 | SaveConfigPage |
| +0x64 | 0x100081c0 | SetDPadParam |
| +0x68 | 0x100083f0 | GetDPadParam |

Identity: slot +0x04 is the SetMacroStep builder that `sapjoy-t3pro.md` names at 0x10005830, and that function builds a SetMacroStep command (observed-static). That supports the table being the ISAPJoyHID vtable (inferred). The RTTI record for the vtable was not decoded, so this is not confirmed.

Slots +0x54 and +0x58 are not among the config exports above. Their targets are not recorded here, and their export identity is unchecked.

## Macro-step commands (observed-static, slots +0x04 and +0x08)

### Command object
Both builders allocate a 0x30-byte `_Ref_count_obj2` block. The command object `C` sits at block +0x0C. Offsets below are relative to `C`:
- `C+0x00`: vtable of the command class (`CSAPJoySetMacroStepCmd` / `CSAPJoyGetMacroStepCmd`).
- `C+0x04`: 16-bit header. `0x0101` for SetMacroStep, `0x0201` for GetMacroStep (both observed-static). The high byte is 0x01 for Set and 0x02 for Get. Wire order is not confirmed (inferred: little-endian, bytes `01 01` and `01 02`).
- `C+0x08`, `C+0x0C`, `C+0x10`: payload vector begin, end, capacity. Payload is appended through the vector helper.
- `C+0x20`: timeout field, set to 500.

Both builders then take a reference, call the transport helper `0x10012060` with `want_response = 1`, and drop the reference. The transport's return value is discarded.

### SetMacroStep request (observed-static, serializer 0x100020e0)
Input `P` is a 16-byte caller struct. Bytes are emitted in this order, with `m = P[6]`:
1. `P[0..7]` (8 bytes).
2. Mode block, chosen by `m`:
   - `m = 0`: four `0x00` bytes. `P[8..11]` and `P[10..11]` are not sent.
   - `m = 1`: `P[8..11]` (4 bytes).
   - `m = 2`: `P[8]`, `P[9]`, then `P[10]`, `P[11]`.
   - `m = 3` or `4`: two `0x00` bytes, then `P[10]`, `P[11]`.
   - any other `m`: nothing. `P[10..11]` are not sent.
3. `P[12..15]` (4 bytes), in every mode.

Payload length: 16 bytes for `m` = 0, 1, 2, 3, 4. 12 bytes for any other `m`. The 2-byte header is not part of this payload.

### SetMacroStep reply check (observed-static, builder 0x10005830)
The builder calls the command's virtual at `C` slot +0x24 with no output. By default that slot is `0x10001de0` (the response check in `sapjoy-t3pro.md`). The builder maps the result:
- `-1` → `0xFFFFFFFB`
- `0` → `0`
- `1` → `0xFFFFFFF5`
- other → `0xFFFFFFFD`

On a non-zero result it logs "Execute SetMacroStep command failed: 0x%x".

### GetMacroStep request (observed-static, builder 0x100059a0)
Payload is two bytes: the caller's first argument, then its second. The builder takes three arguments; the third is the output pointer.

### GetMacroStep reply parse (observed-static, 0x1000a770)
The parser reads the response buffer through `C` slot +0x18, which returns a begin/end pair `R`. With `n = end - begin`:
- If `n > 4` and (`R[2] != 0` or `R[3] != 0`): copy `R[4..19]` (16 bytes) to the output and return 0.
- Otherwise log "Command{ 0x%02X, 0x%02X } not supported" and return -1.

Two notes from the code, both observed-static:
- The parser does not check `n >= 20` before reading `R[4..19]`. A short reply reads past the buffer.
- It does not treat `R[4]` as a status byte. That differs from `0x10001de0`, which the Set path uses.

If the output pointer is null, the parser returns 0 and copies nothing. If `R+4` is null, it zeroes the output and sets errno to EINVAL.

The GetMacroStep builder maps the parser result:
- `-1` → `0xFFFFFFFB`
- `0` → `0`
- `1` → `0xFFFFFFF4`
- other → `0xFFFFFFFD`

On a non-zero result it logs "Execute GetMacroStep command failed: 0x%x".

Note the difference from Set: Get maps `1` to `0xFFFFFFF4`, Set maps it to `0xFFFFFFF5`.

## Not yet decoded
- Get/Set payload layouts for: StickCurveHSV, StickMapping, TriggerParam, MotionSenseCurveHSV, MotionSenseMapping, KeyMappingHSV, VibrateLevel, LightParam, LightColorHSV, DPadParam. Slot targets are in the table above.
- SaveConfigPage frame (slot +0x5c, 0x10007f00).
- Identity and targets of slots +0x54 and +0x58.
- RTTI record for the vtable. Not confirmed.
- Wire framing of the header plus payload: whether `0x10012060` writes these bytes as-is or pads to a 64-byte report. Not read.
- Argument fill values used by the vendor app, and the cross-check against the command definitions in the vendor SDK header.
