/** MOK in block characters: two rows torn sideways and a few cells decayed. */
export const INK = [
  "  ██   ██  █████  ██   ██ ░",
  "  ███ ███ ███░███ ██  ██   ",
  "░▒  ███████ ██   ██ ██ ██ ░",
  "  ██ █ ██ ██   ██ ████     ",
  "  ██   ██ ██   ▒█ ██ ██    ",
  "░██   ██ ███ ███ ██  ██   ▒",
  "▒ ██   ██  █████  ██   ██  ",
];

/** The torn rows once more, a few cells off, for the accent to show through. */
export const GHOST = [
  "                           ",
  "                           ",
  " ███████ ██   ██ ██ ██     ",
  "                           ",
  "                           ",
  "    ██   ██ ███ ███ ██  ██ ",
  "                           ",
];

export const WIDTH = INK[0].length;
export const HEIGHT = INK.length;

/** How much of its cell each character fills. */
const FILL: Record<string, number> = { "█": 1, "▒": 0.5, "░": 0.25 };

export interface Run {
  x: number;
  y: number;
  width: number;
  opacity: number;
}

/** Neighboring cells of one fill, as one rectangle. */
export function runs(rows: string[]): Run[] {
  const out: Run[] = [];
  rows.forEach((row, y) => {
    let x = 0;
    while (x < row.length) {
      const opacity = FILL[row[x]];
      if (opacity === undefined) {
        x++;
        continue;
      }
      let end = x;
      while (end < row.length && FILL[row[end]] === opacity) end++;
      out.push({ x, y, width: end - x, opacity });
      x = end;
    }
  });
  return out;
}
