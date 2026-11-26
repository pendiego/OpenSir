# Cyclone 2 input report (gamepad state)

Status: observed-static from the vendor app's behaviour. The parser reads byte offsets relative to the buffer passed to it. [INFERENCE] whether that buffer includes the leading `0x0A` inbound header is not confirmed; check against a captured device report.

## Byte map (as read by the parser)

| Offset | Field | Decode |
|---|---|---|
| 1 | ls.x | `v-128` scaled to -1..1, rounded to 1e-4 |
| 2 | ls.y | same |
| 3 | rs.x | same |
| 4 | rs.y | same |
| 5, low nibble | Directional (D-pad) | 0 up, 1 up-right, 2 right, 3 right-down, 4 down, 5 down-left, 6 left, 7 left-up; 8 = none |
| 5 bit 4 | SQU | |
| 5 bit 5 | CRO | |
| 5 bit 6 | CIR | |
| 5 bit 7 | TRI | |
| 6 bit 0 | L1 | |
| 6 bit 1 | R1 | |
| 6 bit 2 | L2 | |
| 6 bit 3 | R2 | |
| 6 bit 4 | Share | |
| 6 bit 5 | Options | |
| 6 bit 6 | L3 | |
| 6 bit 7 | R3 | |
| 7 bit 0 | Home | |
| 7 bit 1 | Camera | |
| 7 bit 3 | FL1 | overwritten by the push handler, see below |
| 7 bit 4 | FR1 | overwritten by the push handler, see below |
| 7 bit 5 | M | overwritten by the push handler, see below |
| 8 | LeftTrigger | raw 0..255 |
| 9 | RightTrigger | raw 0..255 |

## FL1, FR1, M (observed-static)
The vendor push handler parses the full report, then parses bytes 53..62 as a second report and replaces `FL1`, `FR1`, `M` with values from that second parse. So those three flags come from bytes 53..62 of the buffer, where slice index 7 is buffer offset 60. Bit 3 at offset 60 is FL1, bit 4 is FR1, bit 5 is M.

## Not verified here
- Battery, charge state, config index, and RGB fields. An earlier note said bytes 35–36 for battery; this was not confirmed by the parser. Treat as open.
- Whether the buffer includes the `0x0A` header. If it does, every offset above shifts by 1.

## Open questions
- Battery and charge offsets (not read by the main parser; check the push and RGB read paths).
- Header offset (see note at top).
- No hardware sample decoded here; validate against a captured device report before relying on it.

## Open question: battery bytes

Battery uses vendor offsets: `report[35]` is the charge state (1 = charging) and `report[36]` is the level, clamped to 100.
Sample: `[35]=0x4b [36]=0x03`. Byte 35 is 0x4b, not 0/1, so its meaning is unclear and the charging flag is probably wrong for this sample. Needs a capture while charging and while not charging.
