// Writes public/favicon.svg, public/apple-touch-icon.png and the repo's logo.png from the header's mark.
import { chromium } from "@playwright/test";
import { readFileSync, writeFileSync } from "node:fs";
import { GHOST, HEIGHT, INK, WIDTH, runs } from "../src/wordmark.ts";

const css = readFileSync("src/styles.css", "utf8");

/** The colors a `:root` block of styles.css sets, as the header's mark wears them. */
function colors(selector) {
  const start = `\n${css}`.indexOf(`\n${selector} {`);
  const block = start < 0 ? null : css.slice(start, css.indexOf("}", start));
  if (!block) throw new Error(`favicon: styles.css has no ${selector} block`);
  const v = (name) => {
    const value = block.match(new RegExp(`--${name}: (#[0-9a-f]{6});`))?.[1];
    if (!value) throw new Error(`favicon: ${selector} sets no --${name}`);
    return value;
  };
  return { back: v("bg"), ink: v("iso-line"), accent: v("accent") };
}
const [LIGHT, DARK] = [colors(":root"), colors(":root.dark")];

const rules = (c) => `.ink{fill:${c.ink}}.ghost{fill:${c.accent}}`;

/** Columns `from` to `to` of the mark, centered in a square with `inset` of the side left round it. */
function svg({ from = 0, to = WIDTH, inset, style, back = "" }) {
  const width = to - from;
  const side = Math.max(width, HEIGHT) / (1 - inset * 2);
  const [ox, oy] = [(side - width) / 2, (side - HEIGHT) / 2];
  const rects = (rows) =>
    runs(rows.map((row) => [...row].slice(from, to).join("")))
      .map((r) => `<rect x="${+(r.x + ox).toFixed(3)}" y="${+(r.y + oy).toFixed(3)}" width="${r.width}" height="1"${r.opacity < 1 ? ` opacity="${r.opacity}"` : ""}/>`)
      .join("");
  const fill = back && `<rect width="${+side.toFixed(3)}" height="${+side.toFixed(3)}" fill="${back}"/>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${+side.toFixed(3)} ${+side.toFixed(3)}" shape-rendering="crispEdges"><style>${style}</style>${fill}<g class="ghost" opacity=".85">${rects(GHOST)}</g><g class="ink">${rects(INK)}</g></svg>\n`;
}

// The small icons show the M alone; the wordmark does not read at 16 pixels.
const M = { from: 0, to: 11 };
writeFileSync("public/favicon.svg", svg({ ...M, inset: 0.04, style: `${rules(LIGHT)}@media (prefers-color-scheme:dark){${rules(DARK)}}` }));

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 180, height: 180 } });
await page.setContent(`<style>body{margin:0}svg{display:block;width:180px;height:180px}</style>${svg({ ...M, inset: 0.1, style: rules(LIGHT), back: LIGHT.back })}`);
await page.screenshot({ path: "public/apple-touch-icon.png" });
await page.setViewportSize({ width: 2048, height: 2048 });
await page.setContent(`<style>body{margin:0;background:none}svg{display:block;width:2048px;height:2048px}</style>${svg({ inset: 0.04, style: rules(LIGHT) })}`);
await page.screenshot({ path: "../logo.png", omitBackground: true });
await browser.close();
console.log("favicon: wrote public/favicon.svg, public/apple-touch-icon.png and ../logo.png");
