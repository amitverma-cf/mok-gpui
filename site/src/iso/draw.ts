import { bounds, faces, inside, painted, rise, type Block, type Point } from "./engine";
import { BEAT, strike } from "./marimba";

const DRAW = BEAT * 2;
const GROW = 320;
const BUMP = 520;

interface Ink {
  ground: string;
  line: string;
  accent: string;
  side: string;
  shade: string;
}

function ink(): Ink {
  const css = getComputedStyle(document.documentElement);
  const v = (name: string) => css.getPropertyValue(name).trim();
  return { ground: v("--bg"), line: v("--iso-line"), accent: v("--accent"), side: v("--iso-side"), shade: v("--iso-shade") };
}

/** Grows cell by cell; a pointed block lifts and sounds. */
export function miniature(canvas: HTMLCanvasElement, model: Block[], still: boolean, onHover?: (index: number | null) => void) {
  const blocks = painted(model);
  const ctx = canvas.getContext("2d")!;
  let colors = ink();
  let started: number | null = still ? -Infinity : null;
  let struck = -1;
  let frame = 0;
  let view = { scale: 1, dx: 0, dy: 0 };
  const bumps = new Map<Block, number>();
  // Heights as last drawn; only grown blocks answer the pointer.
  const shown = new Map<Block, number>();

  const fit = () => {
    const dpr = Math.min(devicePixelRatio, 2);
    const { clientWidth: w, clientHeight: h } = canvas;
    canvas.width = Math.round(w * dpr);
    canvas.height = Math.round(h * dpr);
    const [x0, y0, x1, y1] = bounds(blocks);
    const scale = Math.min((w * 0.9) / (x1 - x0), (h * 0.86) / (y1 - y0));
    view = { scale: scale * dpr, dx: (w * dpr - (x1 + x0) * scale * dpr) / 2, dy: (h * dpr - (y1 + y0) * scale * dpr) / 2 };
  };

  const at = ([x, y]: Point): Point => [x * view.scale + view.dx, y * view.scale + view.dy];

  const path = (poly: Point[], upto = 1) => {
    ctx.beginPath();
    const pts = poly.map(at);
    ctx.moveTo(...pts[0]);
    // The footprint draws itself edge by edge.
    const edges = pts.length * upto;
    for (let i = 1; i <= Math.floor(edges); i++) ctx.lineTo(...pts[i % pts.length]);
    const part = edges - Math.floor(edges);
    if (part > 0 && edges < pts.length) {
      const [a, b] = [pts[Math.floor(edges) % pts.length], pts[(Math.floor(edges) + 1) % pts.length]];
      ctx.lineTo(a[0] + (b[0] - a[0]) * part, a[1] + (b[1] - a[1]) * part);
    }
  };

  const draw = (now: number) => {
    frame = 0;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.lineJoin = "round";
    ctx.lineWidth = Math.max(1, view.scale * 0.018);
    let moving = false;
    shown.clear();
    for (const b of blocks) {
      const since = started === null ? -1 : now - started - b.beat * BEAT;
      if (since < 0) {
        moving ||= started !== null;
        continue;
      }
      const drawn = Math.min(1, since / DRAW);
      const grown = rise((since - DRAW) / GROW);
      const bump = bumps.get(b);
      const bounce = bump === undefined ? 0 : Math.max(0, 1 - (now - bump) / BUMP);
      if (bump !== undefined && bounce === 0) bumps.delete(b);
      moving ||= drawn < 1 || since - DRAW < GROW || bounce > 0;
      const h = Math.max(b.h * grown * (1 + 0.35 * Math.sin(bounce * Math.PI)), 0.0001);
      const f = faces(b, h);
      if (drawn === 1) shown.set(b, h);
      const stroke = b.accent ? colors.accent : colors.line;
      if (drawn < 1) {
        ctx.strokeStyle = stroke;
        path(f.top, drawn);
        ctx.stroke();
        continue;
      }
      for (const [poly, fill] of [[f.right, colors.shade], [f.front, colors.side], [f.top, colors.ground]] as const) {
        path(poly);
        ctx.closePath();
        ctx.fillStyle = fill;
        ctx.fill();
        ctx.strokeStyle = stroke;
        ctx.stroke();
      }
    }
    if (started !== null && !still) {
      const beat = Math.floor((now - started) / BEAT);
      for (; struck < beat; ) {
        struck++;
        const first = blocks.find((b) => b.beat === struck);
        if (first) strike(struck + Math.round(first.h * 2));
      }
    }
    if (moving) request();
  };

  const request = () => {
    if (!frame) frame = requestAnimationFrame(draw);
  };

  /** The grown block at a point, by model index. */
  const pick = (x: number, y: number): number | null => {
    const r = canvas.getBoundingClientRect();
    const dpr = canvas.width / r.width;
    const p: Point = [((x - r.left) * dpr - view.dx) / view.scale, ((y - r.top) * dpr - view.dy) / view.scale];
    for (let i = blocks.length - 1; i >= 0; i--) {
      const b = blocks[i];
      const h = shown.get(b);
      if (h === undefined) continue;
      const f = faces(b, h);
      if (inside(p, f.top) || inside(p, f.front) || inside(p, f.right)) return b.y === 0 ? null : model.indexOf(b);
    }
    return null;
  };

  const lift = (index: number) => {
    const b = model[index];
    if (still || bumps.has(b)) return;
    bumps.set(b, performance.now());
    strike(Math.round(b.x + b.z + b.h * 2), 0.5);
    request();
  };

  let hovered: number | null = null;
  const hover = (at: number | null) => {
    if (at === hovered) return;
    hovered = at;
    if (at !== null) lift(at);
    onHover?.(at);
  };
  const pointer = (e: PointerEvent) => hover(pick(e.clientX, e.clientY));
  const leave = () => hover(null);

  const resized = new ResizeObserver(() => (fit(), request()));
  resized.observe(canvas);
  canvas.addEventListener("pointermove", pointer);
  canvas.addEventListener("pointerleave", leave);
  fit();
  request();
  return {
    pick,
    lift,
    start() {
      // Still, it stands grown from the first frame.
      if (started !== null) return;
      started = performance.now();
      struck = -1;
      request();
    },
    replay() {
      if (still) return;
      shown.clear();
      started = performance.now();
      struck = -1;
      request();
    },
    repaint() {
      colors = ink();
      request();
    },
    stop() {
      cancelAnimationFrame(frame);
      resized.disconnect();
      canvas.removeEventListener("pointermove", pointer);
      canvas.removeEventListener("pointerleave", leave);
    },
  };
}

