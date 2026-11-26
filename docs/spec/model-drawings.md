# Controller drawings

The controller drawing is GameSir Connect's official cover, one per model, stored at `static/pads/<id>.svg` (user decision, Nov 2026: the vendor artwork is bundled). OpenSir places its hotspots, chips and live highlights on top of the cover (`src/lib/components/Controller.svelte`); the device list shows the same cover as a thumbnail (`PadThumb.svelte`).

## Sources

The covers are GameSir Connect's controller artwork (the theme folder names below are the vendor's). One theme per model is bundled, minified with `npx svgo@3 -p 2` (`-p 1` for T3CE and T3Pro, whose camo/dot patterns made them 0.6 and 2.5 MB):

| id | Theme folder | viewBox (w × h) | Frame height |
|---|---|---|---|
| C2 | `C2/White` | 441.75 × 291.82 | 660.6 |
| T3CE | `T3CE/Default` | 460.79 × 284.14 | 616.63 |
| G7ProCE | `G7ProCE/G7Pro` | 430.53 × 292.13 | 678.54 |
| K2_8K | `K2_8K/Black` | 442.4 × 292 | 660.04 |
| T3Lite | `T3Lite/White` | 461.88 × 280.1 | 606.44 |
| T3Pro | `T3Pro/Black` (also used for the T3Pro/T4NPro dongle) | 460.79 × 284.14 | 616.63 |
| T4NPro | `T4NPro/Black` | 439.55 × 295.57 | 672.43 |
| T4N2 | `T4N2/White` | 439.87 × 295.92 | 672.74 |
| N2S | `N2S/White` | 440.16 × 296.21 | 672.96 |
| T3XS | `T3XS/White` | 461.88 × 280.1 | 606.44 |

Unknown models use the C2 cover in greyscale (`mono`).

## Frame

- Each cover is drawn at `<image x=0 y=0 width=1000 height=h>`, `h = 1000 × viewBox height / width`.
- The SVG viewBox is `0 -60 1000 (h + 145)`: 60 units above the cover for the trigger chips, 145 below for back-key chips.
- All coordinates in `src/lib/pads/<id>.json` are in this 1000-wide frame.

## Per-model data (`src/lib/pads/<id>.json`, loaded by `src/lib/models.ts`)

- `height`: frame height of the cover.
- `spots`: one hotspot per key id of `src/lib/keys.ts` (`L2`, `L1`, `L3`, `Up`, `Under`, `Left`, `Right`, `TRI`, `SQU`, `CIR`, `CRO`, `R1`, `R2`, `R3`, `PS`, `Sel`, `Sta`, `Camera`, `FL1`, `FR1`). Each has `x`, `y`, `shape` (`circle` with `r`, or `rect` with `w`, `h`, `rx`, optional `rot` in degrees) and an optional `chip`.
- `extra`: spots for `ModelInfo.extraKeys` in order (G7ProCE FL2/FR2; K2_8K FL2/FR2/FL3/FR3; T3CE Ext1–Ext7).
- `touchKeys`: T3CE Ext8/Ext9 (left/right half of the black top plate).
- `leds`: C2 grip light strips (slots 0 and 1) as ellipses; the home glow (slot 3) is drawn around the `PS` spot.

## Chips

`chip: true` marks a key that the cover does not show; the Controller draws a labelled pill for it.

- L2/R2 on every model: `rect 120×30`, `y = -32`, `x` = the L1/R1 spot.
- Back keys: `rect 96×32`, `y = round(h) + 55`, spaced so pills do not overlap (FL1/FR1 at 400/600 unless more keys share the row).

## Judgement calls (from the covers, not confirmed on hardware)

- Small centre keys: View (`Sel`) = the two-squares or "−" key, Menu (`Sta`) = the "≡" or "+" key, Share (`Camera`) = the capture key.
- K2_8K: View/Menu are the lower angled keys beside the centre plate (they match the vendor `View.png`/`Options.png`); the upper ridged angled keys are FL1/FR1 (L4/R4); FL2/FR2 and FL3/FR3 (vendor L5/R5, L6/R6) are back chips.
- T3CE: C1–C4 (Ext1–Ext4) are the printed pills on the front; Ext5–Ext7 are chips; the touch keys sit on the black top plate.
- T3Lite/T3XS: the "HOME" pill is `PS`, the "CAPTURE" pill is `Camera`.
- N2S: the cover shows no Home/View/Menu/Share key (the top centre is a screen), so they are chips.

## Measuring

A local overlay script (not part of this repository) renders the cover in the frame with a 25-unit grid and the spots of the JSON (red = on cover, blue = chip). Edit the JSON, render, look, repeat. Run renders one at a time (they share a temp HTML file per model).
