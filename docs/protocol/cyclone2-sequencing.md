# Cyclone 2 write and read sequencing

Status: observed-static from the vendor app's behaviour unless marked. Transport timing details are in `cyclone2-transport.md`.

## Framing (answer)
- Each outbound packet is 64 bytes: the command builder fills an array of 64 from index 0 (observed-static).
- Byte 0 is `0x0F` for every outbound command. There is no leading report-ID byte in the 64-byte payload that the code builds.
- Bytes after the payload are zero (array default).
- Inbound packets start with `0x0A`; class is byte 1 (observed-static).
- Transport doc records that a leading `0x00` report ID returns -1 from hidapi on this device. Use 64 bytes starting with `0x0F`.

## Write chunk
Header (6 bytes) + payload, total 64 bytes.

| Index | Value |
|---|---|
| 0 | 0x0F |
| 1 | 3 (WriteProfile) |
| 2 | slot (32 light, 5 shift, else profile number) |
| 3 | offset high byte |
| 4 | offset low byte |
| 5 | payload length |
| 6.. | payload |

Max payload per chunk: `EveryOutPacketLength` = 58 bytes. Max total packet: 64 bytes. Max write chunk length: 58 payload bytes.

## Write procedure (observed-static)
1. `writeProfile(model, slot)` encodes with the light encoder (slot "Light") or the 680-byte profile encoder (other slots).
2. The new blob is diffed against the stored `profileBytes`. Only the span from `StartDiffIndex` to `EndDiffIndex` is written. If both are -1 nothing is sent.
3. The changed span is split into chunks of up to 58 bytes. Each chunk carries its offset in the slot.
4. Each chunk is sent in order through the ack-waiting write routine. That routine queues, resends after 200 ms, and gives up after about 10 sends (transport doc).

Chunks are sent in ascending offset order. Each waits for an ack before the next.

## Enter and exit config
`getEnterProfileConfigCommand` = `[15, 1]`; `getExitProfileConfigCommand(flag)` = `[15, 2, flag ? 1 : 0]`. These are defined but not called from the profile write path. [INFERENCE] they belong to the firmware-update path, which is out of scope.

## Switch
`getSwitchProfileCommand` = `[15, 7, slot]`; `"Shift"` maps to 5. Sent when the profile is switched.

## Light status
`getSetLightStatusCommand(on, t)` = `[15, 13, on ? 1 : 0, t]`. Sent immediately without waiting for an ack. Same id as ReadRGB (`[15, 13]`).

## Read procedure
- `getReadProfileCommand` sends chunked `[15, 4, n, off_hi, off_lo, len]`.
- Chunks are 58 bytes. Total 635 for slot 32, 680 for others.
- On completion the code emits a read-complete event.

## Open questions
- Whether hardware requires enter-config before WriteProfile. Static code does not call it; no hw test was run (non-config mode).
- Ack timing values come from `cyclone2-transport.md`; not re-verified here.
