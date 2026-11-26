import { flushSync } from "svelte";

/**
 * Applies a state change that swaps a whole screen (device list ⇄ device, theme) inside a
 * same-document view transition: the browser crossfades the two states and morphs elements
 * that share a `view-transition-name` (the controller art). Without support, or with reduced
 * motion, the change is applied directly.
 * Resolves once `update` has run (not when the animation ends), so callers keep their ordering.
 */
export function withViewTransition(update: () => void): Promise<void> {
  if (typeof document === "undefined" || !document.startViewTransition || matchMedia("(prefers-reduced-motion: reduce)").matches) {
    update();
    return Promise.resolve();
  }
  return document.startViewTransition(() => flushSync(update)).updateCallbackDone;
}
