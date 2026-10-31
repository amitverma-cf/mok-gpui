import { useEffect, useRef, useSyncExternalStore } from "react";
import { useMode } from "../theme";
import { miniature } from "./draw";
import type { Block } from "./engine";

const REDUCED = matchMedia("(prefers-reduced-motion: reduce)");

/** Whether the system asks for reduced motion, kept current. */
export function useStill(): boolean {
  return useSyncExternalStore(
    (listen) => (REDUCED.addEventListener("change", listen), () => REDUCED.removeEventListener("change", listen)),
    () => REDUCED.matches,
  );
}

interface Props {
  model: Block[];
  label: string;
  className?: string;
  onHover?: (index: number | null) => void;
  onPick?: (index: number) => void;
  lift?: number | null;
}

/** Grows once shown; a press regrows or picks. */
export function Iso({ model, label, className, onHover, onPick, lift }: Props) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const mini = useRef<ReturnType<typeof miniature> | null>(null);
  const hover = useRef(onHover);
  hover.current = onHover;
  const theme = useMode();
  const still = useStill();

  useEffect(() => {
    const m = miniature(canvas.current!, model, still, (i) => hover.current?.(i));
    mini.current = m;
    const seen = new IntersectionObserver(([e]) => e.isIntersecting && m.start(), { threshold: 0.35 });
    seen.observe(canvas.current!);
    return () => {
      seen.disconnect();
      m.stop();
    };
  }, [model, still]);

  useEffect(() => mini.current?.repaint(), [theme]);
  useEffect(() => {
    if (lift != null) mini.current?.lift(lift);
  }, [lift]);

  const press = (x: number, y: number) => {
    const at = onPick && mini.current?.pick(x, y);
    if (at != null) onPick!(at);
    else mini.current?.replay();
  };
  const name = `iso ${className ?? ""}`;
  // Still, nothing replays; picks stay the pointer's.
  if (still) return <canvas ref={canvas} className={name} role="img" aria-label={label} onClick={(e) => press(e.clientX, e.clientY)} />;
  return (
    <button className={name} aria-label={`${label}. Grow it again`} onClick={(e) => (e.detail ? press(e.clientX, e.clientY) : mini.current?.replay())}>
      <canvas ref={canvas} />
    </button>
  );
}
