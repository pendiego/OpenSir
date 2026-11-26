// Single import point for UI code. Wire types come from `types.ts` (CoreModels' mirror of
// crates/opensir-core/src/model.rs); only UI-side aliases live here.
export * from "./types";

export type Rgb = [number, number, number];
/** One light frame: RGB0..RGB4 in device slot order. */
export type Frame = [Rgb, Rgb, Rgb, Rgb, Rgb];
