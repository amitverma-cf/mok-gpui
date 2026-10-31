import type { Block } from "./engine";

const b = (x: number, z: number, w: number, d: number, h: number, y = 0, accent = false): Block => ({ x, z, w, d, h, y, accent, beat: 0 });

/** Beats from the far corner toward the eye. */
const sweep = (blocks: Block[], pace: number) => blocks.map((k) => ({ ...k, beat: Math.round((k.x + k.z) * pace) }));

/** An app: title bar, sidebar, chart, chat, switch. */
export function app(): Block[] {
  const s: Block[] = [b(0, 0, 18, 12, 0.5)];
  s.push(b(0.6, 0.6, 16.8, 1, 0.25, 0.5));
  for (let i = 0; i < 3; i++) s.push(b(1 + i * 0.8, 0.85, 0.45, 0.45, 0.25, 0.75, i === 0));
  for (let i = 0; i < 6; i++) s.push(b(0.8, 2.2 + i * 1.5, 3.4, 1, 0.25, 0.5, i === 1));
  s.push(b(5, 2.2, 7.4, 1, 0.2, 0.5));
  s.push(b(5, 3.8, 7.4, 6.4, 0.08, 0.5));
  [0.9, 1.5, 1.2, 2.1, 1.7, 2.8, 2.3, 3.2].forEach((h, i) => s.push(b(5.5 + i * 0.85, 7.6, 0.55, 1.2, h, 0.58, i === 5)));
  for (let i = 0; i < 4; i++) s.push(b(i % 2 ? 14.2 : 13, 2.2 + i * 1.7, i % 2 ? 2.8 : 3.6, 1.2, 0.4, 0.5, i === 3));
  s.push(b(13, 9.8, 1.6, 0.8, 0.3, 0.5), b(13.9, 9.8, 0.7, 0.8, 0.6, 0.5, true));
  return sweep(s, 0.9);
}
/** Twenty candles over their volume. */
export function candles(): Block[] {
  const s: Block[] = [b(0, 0, 21, 8, 0.3)];
  const days: Array<[number, number, number, number]> = [];
  let close = 0;
  for (let i = 0; i < 20; i++) {
    const open = close;
    close = open + Math.sin(i * 1.7) * 1.1 + Math.cos(i * 0.6) * 0.6 + 0.25;
    days.push([open, close, Math.min(open, close) - 0.4 - (i % 3) * 0.2, Math.max(open, close) + 0.35 + (i % 2) * 0.3]);
  }
  const floor = Math.min(...days.map((d) => d[2]));
  const range = Math.max(...days.map((d) => d[3])) - floor;
  const lift = (price: number) => 0.9 + ((price - floor) / range) * 4;
  days.forEach(([open, close, low, high], i) => {
    const x = 0.7 + i;
    const [lo, hi] = [lift(Math.min(open, close)), lift(Math.max(open, close))];
    s.push({ ...b(x + 0.33, 2.73, 0.14, 0.14, lift(high) - lift(low), lift(low)), beat: 2 + i });
    s.push({ ...b(x, 2.4, 0.8, 0.8, Math.max(hi - lo, 0.2), lo, close >= open), beat: 2 + i });
    s.push({ ...b(x, 5, 0.8, 1.6, 0.2 + ((i * 7) % 5) * 0.22, 0.3), beat: 3 + i });
  });
  return s;
}

/** Five series of eight months, one in the accent. */
export function bars(): Block[] {
  const s: Block[] = [b(0, 0, 11, 8, 0.3)];
  for (let z = 0; z < 5; z++) {
    for (let x = 0; x < 8; x++) {
      const h = 0.4 + 2.4 * (0.5 + 0.5 * Math.sin(x * 0.7 + z * 1.1)) * (1 + z * 0.12);
      s.push({ ...b(1 + x * 1.2, 1 + z * 1.3, 0.8, 0.8, h, 0.3, z === 2), beat: 1 + x + z });
    }
  }
  return s;
}

/** A conversation, turn by turn, three dots still typing. */
export function chat(): Block[] {
  const s: Block[] = [b(0, 0, 10, 12, 0.3)];
  const turns: Array<[boolean, number]> = [[false, 6], [true, 4.4], [false, 7.2], [true, 3.2], [false, 5.4]];
  turns.forEach(([mine, w], i) => {
    const z = 1 + i * 2;
    if (!mine) s.push({ ...b(0.8, z + 0.3, 0.8, 0.8, 0.8, 0.3), beat: 1 + i * 3 });
    s.push({ ...b(mine ? 9.2 - w : 2, z, w, 1.4, 0.45, 0.3, mine), beat: 2 + i * 3 });
  });
  for (let i = 0; i < 3; i++) s.push({ ...b(2.2 + i * 0.7, 10.9, 0.45, 0.45, 0.45, 0.3, true), beat: 17 + i });
  return s;
}

/** Indented code beside its gutter, the cursor's line raised. */
export function editor(): Block[] {
  const s: Block[] = [b(0, 0, 14, 11, 0.3)];
  const lines = [6, 8, 5, 0, 9, 7, 10, 4, 0, 8, 6, 3];
  const indent = [0, 1, 2, 0, 0, 1, 2, 3, 0, 1, 1, 0];
  lines.forEach((w, i) => {
    const z = 0.8 + i * 0.8;
    s.push({ ...b(0.7, z, 0.6, 0.45, 0.2, 0.3), beat: 1 + i });
    if (w) s.push({ ...b(2 + indent[i] * 0.8, z, w * 0.9, 0.45, i === 6 ? 0.7 : 0.25, 0.3, i === 6), beat: 1 + i });
  });
  return s;
}

/** A month of days, a few holding events. */
export function calendar(): Block[] {
  const s: Block[] = [b(0, 0, 15, 11, 0.3)];
  for (let i = 0; i < 35; i++) {
    const [col, row] = [i % 7, Math.floor(i / 7)];
    const busy = [3, 8, 9, 15, 22, 23, 24, 30].includes(i);
    s.push({ ...b(0.7 + col * 2, 0.7 + row * 2, 1.7, 1.7, busy ? 0.6 + (i % 3) * 0.3 : 0.15, 0.3, i === 23), beat: 1 + col + row });
  }
  return s;
}

/** Every chapter a plinth, as tall as its stories. */
export function city(counts: number[]): Block[] {
  const s: Block[] = [b(0, 0, 25, 16, 0.3)];
  counts.forEach((n, i) => {
    const [cx, cz] = [1 + (i % 9) * 2.6, 1 + Math.floor(i / 9) * 3];
    s.push({ ...b(cx, cz, 1.6, 1.6, 0.4 + n * 0.1, 0.3), beat: 2 + i });
  });
  return s;
}
