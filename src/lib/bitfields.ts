// Wire byte <-> UI value mappings (bit layout per docs/protocol/cyclone2-profile.md; NC = 255 per crates/opensir-core/src/keycodes.rs).

/** Hair mode 0 off, 1 rapid, 2 fixed from quick_trigger_status: bit 7 enabled, bit 1 selects fixed. */
export const hairMode = (status: number): number => (status & 0x80 ? (status & 2 ? 2 : 1) : 0);
export const setHairMode = (status: number, mode: number): number =>
  mode === 0 ? status & 0x7f : 0x80 | (mode === 2 ? 2 : 1);

/** Orientation 0 yaw, 1 roll, 2 combine from active_axis: bit 1 yaw, bit 0 roll, both combine. */
export const orientation = (axis: number): number => ((axis & 3) === 3 ? 2 : axis & 1 ? 1 : 0);
export const setOrientation = (o: number): number => [2, 1, 3][o];

/** 0 and 255 (NC) both mean no key bound. */
export const hasKey = (code: number): boolean => code !== 0 && code !== 255;

/** Wire value written for "no key" (NC, what reset stores). */
export const NO_KEY = 255;
