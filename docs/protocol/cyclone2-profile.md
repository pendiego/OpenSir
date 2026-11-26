# Cyclone 2 profile codec

Byte-level spec of the GameSir Cyclone 2 (model "C2") profile block: the 680-byte
structure the vendor app encodes and decodes. Golden vectors are in
`cyclone2-profile-vectors.json` (same directory).

Scope: profile block layout and field semantics only. Frame/transport framing,
read/write sequencing on the wire, RGB/light and shift profiles, firmware, and UI
are out of scope here.

Labels: `observed-static` (observed from the vendor app's behaviour), `vendor-exec`
(vector produced by running the vendor encoder/decoder), `inferred`. No hardware
evidence is used in this document.

## 1. Profile block

A profile is one 680-byte buffer. Decoding a buffer whose length is not 680 yields
an empty profile (`observed-static`).

| Offset | Size | Field                         |
|-------:|-----:|-------------------------------|
| 0      | 32   | ProfileName (UTF-8, padded)   |
| 32     | 32   | Fun block (section 2)         |
| 64     | 112  | 16 mapping packets x 7 B (section 3), button order in 3.1 |
| 176    | 159  | FL1 function-key packet (section 4) |
| 335    | 159  | FR1 function-key packet (section 4) |
| 494    | 28   | L2 trigger packet (section 5) |
| 522    | 28   | R2 trigger packet (section 5) |
| 550    | 32   | Left stick packet (section 6) |
| 582    | 32   | Right stick packet (section 6) |
| 614    | 33   | Aim motion packet (section 7) |
| 647    | 33   | Tilt motion packet (section 7) |
| 680    | —    | end                           |

Sources: `observed-static` from the order in which the vendor decoder reads the
fields and the encoder writes them back. The offsets are cumulative sizes; `inferred` only in the
sense that they are derived, not stated as constants in the vendor code. The
vectors confirm the offsets byte for byte.

ProfileName: 32 bytes, UTF-8, trimmed on decode (`observed-static`).

## 2. Fun block (offset 32, 32 bytes)

Decode order (`observed-static`):

| Rel. | Field                    | Notes |
|-----:|--------------------------|-------|
| 0    | motor left               | level 0-4 (section 9.1) |
| 1    | motor right              | level 0-4 |
| 2    | motor lt                 | level 0-4 |
| 3    | motor rt                 | level 0-4 |
| 4    | audio profile_audio_en   | |
| 5    | audio_volume             | |
| 6    | audio_mixer              | |
| 7    | mic_mute                 | |
| 8    | mic_sensitivity          | |
| 9    | shift_en                 | |
| 10   | shift_value              | |
| 11   | dpad_diagonal_lock_en    | |
| 12   | xinput_abxy_change       | |
| 13   | switch_abxy_change       | |
| 14   | report_rates_gears       | |
| 15-31| reserved                 | copied back verbatim on encode |

Reserved bytes survive a round trip because the encoder writes into the decoded
`_RawProfile` copy and the reserved region is not touched (`observed-static`).
The app profile carries `_RawProfile` for this reason.

## 3. Mapping packet (7 bytes each)

16 packets, one per button, in this order (`observed-static`):

Up, Under, Left, Right, L1, R1, L3, R3, CRO, CIR, SQU, TRI, PS, Sel, Sta, Camera.

Packet layout (`observed-static`):

| Rel. | Field        | Notes |
|-----:|--------------|-------|
| 0    | turbo_en     | bit 0 used |
| 1    | turbo_speed  | turbo rate; decoded as 0 when turbo_en is clear |
| 2    | map_en       | 1 = mapped |
| 3    | map_value1   | key code (section 9.4) |
| 4    | map_value2   | key code, 0 if unused |
| 5    | map_value3   | key code, 0 if unused |
| 6    | toggle_en    | bit 0 = toggle on press |

Decode key typing (`observed-static`):
- `turbo` = turbo_speed when turbo_en bit 0 is set, otherwise 0.
- `mapKeys` = non-empty names of map_value1..3 when map_en bit 0 is set.
- `keyType`/`keyName`: one key containing `_` gives "Keyboard"; one key without `_` gives "Mapped";
  more than one gives "More"; none gives "Self".
- `toggleOnPress` = toggle_en bit 0.

Vector evidence (`vendor-exec`): Up mapped to KB_A gives `[0,0,1,92,0,0,0]`;
CRO with turbo 5 gives `[1,5,0,0,0,0,0]`; SQU mapped to CRO gives
`[0,0,1,9,0,0,0]`; L1 with toggle gives `[0,0,0,0,0,0,1]`.

## 4. Function-key packet (159 bytes, FL1 and FR1)

Layout (`observed-static`, decoder and encoder):

| Rel.    | Size | Field |
|--------:|-----:|-------|
| 0-1     | 2    | turbo_en, turbo_speed |
| 2-5     | 4    | map_en, map_value1..3 (as 3.) |
| 6       | 1    | toggle_en |
| 7       | 1    | macro flag: bit0 marco_en, bit1 marco_key_open (press-to-trigger), bit2 marco_cycle_en |
| 8-9     | 2    | cycle time, big-endian (high, low), milliseconds |
| 10      | 1    | step_num (0-30 used; decoder clamps to 30) |
| 11-158  | 148  | 30 steps: steps 0-28 are 5 bytes, step 29 is 3 bytes |

Step, 5 bytes: `step_data` (key code), `hold_h`, `hold_l`, `delay_h`, `delay_l`.
Step 29, 3 bytes: `step_data`, `hold_h`, `hold_l` (no delay: the packet ends before it).

Hold and delay: `256*high + low`. A macro is active when marco_en is set and step_num > 0
(`observed-static`).

Vector evidence (`vendor-exec`, vector `macro-fl1`): FL1 at 176 gives header
`[0,0,0,0,0,0,0]` (no turbo, no map, no toggle). Macro header at 183: flag 7,
cycle high 1, cycle low 44 (300 ms), step_num 2. Step 0 at 187: KB_A (92), hold 50,
delay 20. Step 1 at 192: KB_ENTER (103), hold 30, delay 10.

## 5. Trigger packet (28 bytes, L2 and R2)

Layout (`observed-static`, encoder and decoder):

| Rel.  | Size | Field |
|------:|-----:|-------|
| 0-1   | 2    | turbo_en, turbo_speed |
| 2-6   | 5    | dead_en, front_dead, back_dead, anti_front_dead, anti_back_dead |
| 7-10  | 4    | map_en, map_value1..3 |
| 11    | 1    | toggle_en |
| 12    | 1    | quick_trigger_status: bit7 enabled, bit0 mode 1, bit1 mode 2 |
| 13    | 1    | quick_trigger_start_value |
| 14    | 1    | quick_trigger_end_value |
| 15-27 | 13   | linear module (section 8) |

Hair trigger decode (`observed-static`): enabled = bit7 of
quick_trigger_status; mode 1 by default, mode 2 when bit1 is set (bit0 gives mode 1);
begin/end default 10/90 when the packet is absent.

Vector evidence (`vendor-exec`, vector `triggers-hairtrigger-l2-curve-r2`): L2 bytes
match `[0,0,1,10,90,5,95,0,0,0,0,0,130,20,80,1,1,50,0,0,64,43,128,128,191,213,255,255]`.
Quick trigger 130 = bit7 + bit1 (mode 2), begin 20, end 80.

## 6. Stick packet (32 bytes, Left and Right)

Layout (`observed-static`, decoder and encoder):

| Rel.  | Size | Field |
|------:|-----:|-------|
| 0     | 1    | stick_en |
| 1     | 1    | stick_square |
| 2-6   | 5    | dead module: dead_en, front_dead, back_dead, anti_front_dead, anti_back_dead |
| 7-19  | 13   | linear module (section 8) |
| 20-31 | 12   | map module: map_en, x_flip, y_flip, axis_ratio, mouse_dpi, map_index, map_cross, map_up_value, map_down_value, map_left_value, map_right_value, map_dead_value |

Raw enabled (`observed-static`): default true;
when stick_en is set, rawEnabled = stick_square != 0.

map_index is an output-type enum (section 9.3). Decode default for the map module
(`observed-static`): outputType from the output-type enum, reverseHorizontal and
reverseVertical false, cross 0, axisRatio 50, mouseDpi 50, wheel mappings all "".

Vector evidence (`vendor-exec`, vector `sticks-deadzone-curve-wheel-map`): Left stick
bytes start `[1,0, 1,15,85,0,100, 1,2,80, 0,0,32,14,128,99,223,241,255,255, 1,1,1,60,40,3,1,92,103,0,0,92]`.
map_index 3 = Wheel, map_up 92 (KB_A), map_down 103 (KB_ENTER), map_dead 92.

## 7. Motion packet (33 bytes, Aim and Tilt)

Layout (`observed-static`, decoder and encoder):

| Rel.  | Size | Field |
|------:|-----:|-------|
| 0     | 1    | sensor_profile_status (activate behaviour) |
| 1     | 1    | sensor_quick_key_value (button value, section 9.4) |
| 2     | 1    | active_axis (bit field, see below) |
| 3-7   | 5    | dead module |
| 8-20  | 13   | linear module (section 8) |
| 21-32 | 12   | map module (same fields as 6) |

active_axis (`observed-static`, decode and encode):

| Orientation | active_axis bits | byte |
|------------:|------------------|-----:|
| 0           | bit 1            | 2 |
| 1           | bit 0            | 1 |
| 2           | bits 0 and 1     | 3 |

Decode: both bits set gives 2, bit 0 gives 1, bit 1 gives 0. This is the inverse.
Vector `motion-aim-orientation` confirms orientation 1 -> byte 1.

Motion map default: the Aim map in the vector leaves map_index at the default value.

## 8. Linear (curve) module (13 bytes)

Layout, shared by triggers (section 5), sticks (section 6), and motion (section 7):

| Rel. | Field |
|-----:|-------|
| 0    | linear_module_en |
| 1    | linear_status (curve type) |
| 2    | linear_data (curvature) |
| 3-12 | five (original, target) byte pairs, point 1 to point 5 |

Pair 1 is always `(0, 0)`, pair 5 is always `(255, 255)` on encode (`observed-static`,
from the curve setter). Points 2-4 come from
the user's curve points:

- `original_k = round((x - front)/(back - front) * 255)` and `target_k = round((y - anti_front)/(anti_back - anti_front) * 255)`,
  where x and y are the user's point coordinates and the deadzone bounds come from
  section 9.2 (`observed-static`; the anti bounds are used for the target).
- Decode inverse: `value = round(raw/255 * (upper - lower) + lower)` (`observed-static`).

Decode default when the module is absent: type 0, curvature 100, points (25,25),
(50,50), (75,75).

Vector evidence: L2 curve (front 10, back 90) points (30,20) gives original 64 and
target 43, matching the rounding rule above.

## 9. Scalars and enums

### 9.1 Motor levels

App value: level 0-4. Raw byte: `raw = 25 * level` (`observed-static`, encode).
Decode: `level = clamp(round(clamp(raw, 0, 100) / 25), 0, 4)`
(`observed-static`). The same conversion is used for all four motors in the codec.
The vector `name-motors-audio-extend` has
levels 2,4,1,3, giving bytes 50,100,25,75.

### 9.2 Deadzone

Dead module bytes: front, back, anti-front, anti-back (`observed-static`, decode
default `{5, 100, 0, 100}`).

### 9.3 Output types (`observed-static`)

| Name  | Value |
|-------|------:|
| Left  | 1 |
| Right | 2 |
| Wheel | 3 |
| Mouse | 4 |

### 9.4 Button and key codes

Button names (`observed-static`): Up, Under, Left, Right, L1, R1, L3, R3, CRO, CIR,
SQU, TRI, PS, Sel, Sta, Camera, FL1, FR1, L2, R2.

Key codes are a data table mapping names to byte values; lookup by name defaults to 0,
and lookup by value defaults to "" (`observed-static`). Values confirmed by vectors:
KB_A = 92, KB_ENTER = 103, CRO = 9, L1 = 5, Up = 1, Sel = 14. The full table is in
`cyclone2-keycodes.md`.

Note: a mapped slot is written as a button key name only when map_en is 1; otherwise
the name field is empty (`observed-static`).

### 9.5 Enum helpers

- The button enum values equal the names in 9.4. The stick enum is {Left, Right}. The motion enum is {Aim, Tilt}.
- Bit fields: bit t of a byte e is tested as `(e & (1 << t)) != 0` (`observed-static`); the
  encoders set bits the same way.

## 10. Round-trip and sequencing (profile level)

- Encode: `fromModel(app)` writes the raw profile fields into the decoded `_RawProfile`
  copy, so reserved bytes and unknown fields persist.
- Decode: `fromBytes(buf)` reads the 680 bytes in the order of section 1 and returns
  the app profile with `_RawProfile` attached.
- Round trip: `fromBytes(fromModel(p))` equals `p` for all 9 `vendor-exec` vectors and for
  13 of the 21 `vendor-log` blobs. The other 8 are open (section 12).

Transport framing and the order of writes on the wire belong to `cyclone2-transport.md`.

## 11. Vectors

`cyclone2-profile-vectors.json` holds 30 entries, each `{name, source, pid, input_json,
hex, length, decoded_json, roundtrip, error}`. `source` is `vendor-exec` (9 entries,
encoded by running the vendor code; no `pid` key) or `vendor-log` (21 entries, `observed-hw`,
real 680-byte profile blobs from the vendor app's log captured on a real device, `pid` 0x100B on 16 and 0x1053
on 5, taken from the device path in the log). All 30 have length 680 and error null. The
12 light-profile blobs of 635 bytes are excluded.

Names: default-zero-bytes, name-motors-audio-extend, keyboard-remap-up-and-turbo-cro,
toggle-l1-and-gamepad-map, macro-fl1, triggers-hairtrigger-l2-curve-r2,
sticks-deadzone-curve-wheel-map, motion-aim-orientation, combined-all-fields.

## 12. Open questions

- RGB/light storage: not in the 680-byte profile block; located elsewhere (Light
  profile at index 32). Not covered here.
- Shift profile at index 5 uses this same 680-byte codec (`observed-static`;
  see `cyclone2-shift.md`). Light profile at index 32 uses its own 635-byte codec
  (`cyclone2-light.md`).
- Device PID 0x0575 is not in the C2 `ConfigModePids` table (4125, 4138, 4179, 4107).
  How the vendor app accepts it is open. Not resolved by this document.
- Motion map_index default: the Aim vector leaves it unset; the effective default
  needs confirmation against the app defaults.
- Vendor-log mismatch (`observed-hw`): 8 of 21 blobs do not round-trip (`read-log144`,
  `146`, `344`, `346`, `409`, `411`, `1456`, `1459`; PID 0x100B on six, 0x1053 on two).
  Each differs at exactly 100 bytes, and only in the same sub-fields:
  - byte 1 of every 7-byte mapping slot (offsets 65, 72, ..., 170) and of FL1, FR1, L2,
    R2 (177, 336, 495, 523) holds 0x0a in the device blob while turbo is off; the encoder
    writes 0x00.
  - the L2 and R2 linear modules hold different enable bytes and zeros where the encoder
    writes default curve points (e.g. offset 506: device 0x01, encoded 0x00; offset 511:
    device 0x00, encoded 0x64).
  Inferred cause: the device keeps unused fields in a non-canonical state, and the codec
  canonicalises them. The codec is not byte-exact for disabled sub-blocks. Not fixed; a
  hardware check is needed before changing the encoder.
- PID 0x0575 (the live device in this session, `nativeSupport: false`) does not appear in
  the vendor log. The log shows 0x100B and 0x1053 only.
- Not checked against hardware in this session: the device is in non-config mode and does
  not answer reads. The read-only comparison
  is still pending.
