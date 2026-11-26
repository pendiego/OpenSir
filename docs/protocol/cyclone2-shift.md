# Cyclone 2 shift profile (slot 5)

Status: decoder route observed-static from the vendor app's behaviour. Layout inherited from the 680-byte profile codec, not separately verified against hardware.

## Routing (observed-static)
- Setting the profile maps `"Shift"` to slot 5 (switch command `[15, 7, 5]`).
- Read: the shift profile is read by its profile number (`ShiftProfileNumber`).
- Decode and encode: the shift blob goes through the 680-byte profile codec. The shift blob is 680 bytes, the same as other 680-byte profile slots.
- Write chunks: slot 5 writes use the write-profile command with slot 5.

## Shift length
680 bytes. Length check on read: `256*n + i + a == 680` for slots other than 32.

## Vectors
No separate shift vectors. Shift decoding is the 680-byte profile codec, covered by `cyclone2-profile-vectors.json`, which has 30 entries (see profile doc section 12). Round-trip check fails for 8 vendor-log blobs there. This is documented in the profile doc, not here. `cyclone2-profile.md` agrees that slot 5 uses this codec.

## Open questions
- Shift layout inside the 680 bytes is not separately checked; relies on the profile codec.
- No hw confirmation that slot 5 accepts the 680-byte write without any enter/exit config step.
