# GameSir G7 Pro 8K (G7ProCE)

Profile (1070 B) and light (165 B) layouts, encodings and tests: `hid-family-profile.md` (layouts in `layouts/g7proce-*.json`, vectors in `g7proce-raw-vectors.json`). Transport, commands and input report: `hid-family-transport.md`. This page keeps only what is specific to the G7 Pro 8K.

Labels: `observed-static` (observed from the vendor app's behaviour and configuration), `vendor-exec` (vendor code run in the harness).

## Identity (`observed-static`)
- Product type `G7ProCE`, with its own profile and light codecs and model constants.
- Ten wired/wireless PID pairs, one per colour theme (wired/wireless): Gold 4279/4281, Gold 4301/4302 (restricted, below), Aimlabs 4293/4294, GoldenApple 4349/4350, Nioh 4295/4296, NBA2K 4297/4298, Gold 4299/4300, Crown 4146/4147, Gold 4148/4149, GradientBlue 4347/4348. Usage page 0xFFF0 for all.
- Restricted variant 4301/4302: keyboard/mouse mapping, map-to-more-keys and turbo disabled, mapping tabs limited to buttons, motion page hidden.
- Features: profile rename, profile name length 32.

## Model-specific facts (`vendor-exec`)
- Function keys: FL1, FR1, FL2, FR2 (169 B each, 32 macro steps).
- Fun extend block includes left/right stick anti-jitter (bytes 49, 50).
- Light zones: lower left, upper left, upper right, lower right, home; tail: standby time, dock LED mode, dock LED brightness, report rate.
- Input report: FL2/FR2 at byte 63 bits 0-1 (second parse of bytes 53-63); the G7 parser also passes bytes 26-29 as a 16-bit stick source (meaning unverified).
