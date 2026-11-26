# Cyclone 2 light profile (slot 32, 635 bytes)

Status: layout observed-static from the vendor app's behaviour. Length 635 observed-static and on hardware (12 real captured blobs). Vectors: `cyclone2-light-shift-vectors.json`.

Slot 32 is NOT encoded by the 680-byte profile codec. The write path uses the light encoder when the profile is `"Light"`, and the read path checks length 635 for slot 32.

## Layout (offsets into the 635-byte blob)

| Offset | Size | Field | Notes |
|---|---|---|---|
| 0 | 1 | ConfigIndex | clamped to 0 if > 3 |
| 1 + a*124 | 124 | Animation a (a = 0..4) | 5 animations |

Per animation (relative offset r):

| r | Size | Field | Notes |
|---|---|---|---|
| 0 | 1 | KeyFrameCount | |
| 1 | 1 | EffectCount | |
| 2 | 1 | Speed | stored as `20 - speed`, clamped |
| 3 | 1 | Brightness | clamped to 0..100 |
| 4 + f*15 | 15 | Frame f (f = 0..7) | 5 RGB triples, 3 bytes each |

Tail, offset 621 (14 bytes):

| Offset | Field | Notes |
|---|---|---|
| 621 bit 0 | AudioReactiveMode | |
| 622 | UserEffectIndex | |
| 623-625 | ProfileLed R, G, B | |
| 626 bit 0 | RaiseWakeUp | |
| 627 | StandbyTime | |
| 628-634 | ReservedData | 7 bytes, preserved as-is |

Total: 1 + 5*124 + 14 = 635.

## Round-trip
The round-trip `encode(decode(x)) === x` holds for all 12 real device blobs captured from hardware (source `vendor-log`). The 4 synthetic non-default inputs (source `inferred`) also round-trip. Synthetic inputs are our own test data, not vendor output.

## Write
Sent as WriteProfile chunks on slot 32, diff only (see `cyclone2-sequencing.md`).

## Open questions
- Speed: is `20 - speed` the only transform? Only confirmed from the vendor app's light encoder; no hardware check of the speed scale.
- Brightness range: clamped to 100 in code; the UI maximum is not checked.
- ReservedData: may need to be preserved byte-for-byte; the codec does so.
- Animation count fixed at 5 in code; firmware behaviour for unused animations (KeyFrameCount 0) not verified.
