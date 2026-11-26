# HID family "V2": T3CE, G7ProCE, K2_8K profile and light blocks

Status: implemented (opensir-core `src/profile.rs` `decode_for`/`encode_for`, `effect_light.rs`), tested against vectors produced by running the vendor codecs. No `observed-hw` yet: none of these controllers was available.

Transport, commands and input report: `hid-family-transport.md` (same framing and command bytes as the Cyclone 2; only block lengths differ).

## How the layout was obtained (reproducible)
- A private harness (not published) runs each vendor RAW decoder (`fromBytes` with the raw-to-app converter replaced by identity) on a baseline block, flips one byte at a time and records which raw field changes and how (u8, u16 big/little-endian, bit). Output, facts only: `layouts/{c2,t3ce,g7proce,k2_8k}-profile.json` and `layouts/{t3ce,g7proce,k2_8k}-light.json`. The C2 output matches OpenSir's existing Cyclone 2 offsets, which validates the method.
- A second harness step encodes structured raw objects with the vendor RAW encoder and decodes them back: `{t3ce,g7proce,k2_8k}-raw-vectors.json` (`raw_input`, `hex`, `raw_decoded`). Every case re-encodes byte-identically and `raw_decoded` equals `raw_input`.
- The 16-bit dead zones are big-endian in both directions (decoded `0x1234` from `12 34` and re-encoded to `12 34` by the vendor codec for all three models).

## Profile block
Same section order as the Cyclone 2 (`cyclone2-profile.md`); the sections are wider.

| Section | Cyclone 2 | V2 |
|---|---|---|
| Name | 0, 32 B | same |
| Fun | 32, 32 B: motors 4, audio 5, extend 6 | extend adds sensor quick mode, stick resolution (bytes 47, 48); G7ProCE and K2_8K add left/right stick anti-jitter (49, 50) |
| 16 buttons x 7 B | 64 | same |
| Function keys at 176 | FL1, FR1; 159 B; 30 macro steps | 169 B each; 32 macro steps (5 B per step, the last one has no delay). G7ProCE: FL1 FR1 FL2 FR2. K2_8K: FL1 FR1 FL2 FR2 FL3 FR3. T3CE: FL1 FR1 Ext1..Ext7 |
| Triggers L2, R2 | 28 B | 32 B: dead-zone bounds are u16 big-endian (0..1000) |
| Sticks L, R | 32 B | 36 B: u16 dead-zone bounds |
| Motion Aim, Tilt | 33 B | 41 B: two more quick keys after the first, u16 dead-zone bounds, a Z flip after the Y flip in the map module, and a stick-sensitivity byte at the end |
| Touch keys (T3CE) | - | Ext8, Ext9 after the motion packets, 10 B each: turbo 2, map enable + 3 keys, toggle, trigger mode, sensitivity u16 big-endian |
| Length | 680 | T3CE 1935, G7ProCE 1070, K2_8K 1408 |

Encoder differences from the Cyclone 2: the V2 encoder writes curve points 1 and 5 as given (the Cyclone 2 encoder forces (0,0) and (255,255)).

## Light block (slot 32), "effect" light
Five animations. Each: `mode`, the model's u8 settings, then HSV zones (`h` u16 big-endian, `s`, `v`). After the animations: one byte whose bit 0 is raise-to-wake (other bits kept), then the tail.

| Model | Length | Settings after `mode` | Zones | Tail |
|---|---|---|---|---|
| T3CE | 102 | single-colour, breathing, phantom, rainbow, radar brightness; breathing, phantom, rainbow, radar speed; phantom hue, rainbow hue | panel, home | standby time |
| G7ProCE | 165 | same 11 | lower left, upper left, upper right, lower right, home | standby time, dock LED mode, dock LED brightness, report rate |
| K2_8K | 229 | home effect index, profile-number effect index, home phantom speed, profile-number phantom speed, then the same 11 | lower left, lower right, upper left, upper right, home, profile number, home button | the G7 four, then a 4-byte device id |

## Tests
opensir-core `tests/hid_family.rs`: every raw vector decodes to the vendor's own decode, re-encodes to the vendor bytes, and every byte the vendor decoder reads (except reserved ones, kept in `Profile::raw`) reaches the decoded model. `tests/session.rs` `v2_models_open_and_write_diffs`: a fake device per model opens with the model's block lengths, a no-op save writes nothing, a one-field edit writes only that chunk (u16 big-endian on the wire), and a light edit changes only the edited zone.

## Open
- Hardware confirmation for all three models.
- Meaning of the T3CE extend byte 47/48 values (stick resolution scale) and of the light `mode` numbers beyond their setting names.
