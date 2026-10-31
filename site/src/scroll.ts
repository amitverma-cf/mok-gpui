import Lenis from "lenis";
import "lenis/dist/lenis.css";

let lenis: Lenis | null = null;

/** Smooth scroll for the page; the returned call ends it. */
export function smooth(): { lenis: Lenis; end: () => void } {
  if (lenis) throw new Error("scroll: smooth scroll already runs");
  const own = new Lenis({ autoRaf: true });
  lenis = own;
  return { lenis: own, end: () => (own.destroy(), (lenis = null)) };
}

/** Holds the page still at once, glide included. */
export function hold(still: boolean): void {
  if (still) lenis?.stop();
  else lenis?.start();
}
