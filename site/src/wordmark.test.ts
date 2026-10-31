import assert from "node:assert/strict";
import { test } from "node:test";
import { GHOST, HEIGHT, INK, WIDTH, runs } from "./wordmark.ts";

test("the ink and its ghost are boxes of one size", () => {
  assert.equal(INK.length, HEIGHT);
  assert.equal(GHOST.length, HEIGHT);
  for (const row of [...INK, ...GHOST]) assert.equal([...row].length, WIDTH);
});

test("only the torn rows have a ghost", () => {
  assert.deepEqual(GHOST.map((row) => row.trim() !== ""), [false, false, true, false, false, true, false]);
});

test("neighbors of one fill join into one run, and gaps split them", () => {
  assert.deepEqual(runs(["██ █"]), [
    { x: 0, y: 0, width: 2, opacity: 1 },
    { x: 3, y: 0, width: 1, opacity: 1 },
  ]);
  assert.deepEqual(runs(["█▒░"]).map((r) => r.opacity), [1, 0.5, 0.25]);
});

test("the runs cover exactly the filled cells", () => {
  const cells = INK.join("").replace(/ /g, "").length;
  assert.equal(runs(INK).reduce((sum, r) => sum + r.width, 0), cells);
});
