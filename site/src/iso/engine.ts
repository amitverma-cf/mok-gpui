/** A box on the table, its accent, and its beat. */
export interface Block {
  x: number;
  z: number;
  w: number;
  d: number;
  y: number;
  h: number;
  accent?: boolean;
  beat: number;
}

export type Point = [number, number];

const COS = Math.cos(Math.PI / 6);

/** Isometric: x runs down-right, z down-left, y up. */
export function project(x: number, y: number, z: number): Point {
  return [(x - z) * COS, (x + z) * 0.5 - y];
}

const EPS = 1e-6;

/** Whether `a` paints before `b`: under or behind it. */
function under(a: Block, b: Block): boolean {
  const apartX = a.x + a.w <= b.x + EPS || b.x + b.w <= a.x + EPS;
  const apartZ = a.z + a.d <= b.z + EPS || b.z + b.d <= a.z + EPS;
  if (!apartX && !apartZ) return a.y < b.y || (a.y === b.y && a.h < b.h);
  const [behindX, behindZ] = [a.x + a.w <= b.x + EPS, a.z + a.d <= b.z + EPS];
  // Behind on one axis, before on the other: apart.
  if (apartX && apartZ) return behindX && behindZ;
  return apartX ? behindX : behindZ;
}

/** Back to front: each block after what it covers. */
export function painted(blocks: Block[]): Block[] {
  const n = blocks.length;
  const after: number[][] = blocks.map(() => []);
  const waiting = new Array<number>(n).fill(0);
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      if (i !== j && under(blocks[i], blocks[j])) {
        after[i].push(j);
        waiting[j]++;
      }
    }
  }
  const ready = blocks.map((_, i) => i).filter((i) => waiting[i] === 0);
  const out: Block[] = [];
  while (ready.length) {
    const i = ready.shift()!;
    out.push(blocks[i]);
    for (const j of after[i]) if (--waiting[j] === 0) ready.push(j);
  }
  if (out.length !== n) throw new Error(`iso: ${n - out.length} blocks overlap and have no painting order`);
  return out;
}

/** The three faces the eye sees, at height `h`. */
export function faces(b: Block, h: number): { top: Point[]; right: Point[]; front: Point[] } {
  const [x0, x1, z0, z1, y0, y1] = [b.x, b.x + b.w, b.z, b.z + b.d, b.y, b.y + h];
  return {
    top: [project(x0, y1, z0), project(x1, y1, z0), project(x1, y1, z1), project(x0, y1, z1)],
    right: [project(x1, y0, z0), project(x1, y0, z1), project(x1, y1, z1), project(x1, y1, z0)],
    front: [project(x0, y0, z1), project(x1, y0, z1), project(x1, y1, z1), project(x0, y1, z1)],
  };
}

/** Whether `p` lies inside the convex polygon `poly`. */
export function inside(p: Point, poly: Point[]): boolean {
  let sign = 0;
  for (let i = 0; i < poly.length; i++) {
    const [a, b] = [poly[i], poly[(i + 1) % poly.length]];
    const cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
    if (cross === 0) continue;
    if (sign === 0) sign = Math.sign(cross);
    else if (Math.sign(cross) !== sign) return false;
  }
  return true;
}

/** The projected box that holds every block at full height. */
export function bounds(blocks: Block[]): [number, number, number, number] {
  let [x0, y0, x1, y1] = [Infinity, Infinity, -Infinity, -Infinity];
  for (const b of blocks) {
    const f = faces(b, b.h);
    for (const [x, y] of [...f.top, ...f.right, ...f.front]) {
      [x0, y0, x1, y1] = [Math.min(x0, x), Math.min(y0, y), Math.max(x1, x), Math.max(y1, y)];
    }
  }
  return [x0, y0, x1, y1];
}

/** Back-out easing: a block overshoots its height, then settles. */
export function rise(t: number): number {
  if (t <= 0) return 0;
  if (t >= 1) return 1;
  const u = t - 1;
  return 1 + 2.4 * u ** 3 + 1.4 * u ** 2;
}
