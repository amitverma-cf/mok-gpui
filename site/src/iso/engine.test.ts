import assert from "node:assert/strict";
import { test } from "node:test";
import { bounds, faces, inside, painted, project, rise, type Block } from "./engine.ts";

const block = (x: number, z: number, w: number, d: number, h: number, y = 0): Block => ({ x, z, w, d, h, y, beat: 0 });

test("the projection keeps vertical lines vertical and the axes at thirty degrees", () => {
  const [ox, oy] = project(0, 0, 0);
  const [ux, uy] = project(0, 1, 0);
  assert.equal(ux, ox);
  assert.equal(uy, oy - 1);
  const [xx, xy] = project(1, 0, 0);
  assert.ok(Math.abs(Math.atan2(xy, xx) - Math.PI / 6) < 1e-9);
});

test("a block standing on a slab paints after it", () => {
  const slab = block(0, 0, 10, 10, 0.5);
  const tower = block(2, 2, 1, 1, 3, 0.5);
  assert.deepEqual(painted([tower, slab]), [slab, tower]);
});

test("a block behind another paints first, on either axis", () => {
  const far = block(0, 0, 1, 1, 1);
  const nearX = block(2, 0, 1, 1, 1);
  const nearZ = block(0, 2, 1, 1, 1);
  assert.deepEqual(painted([nearX, far]), [far, nearX]);
  assert.deepEqual(painted([nearZ, far]), [far, nearZ]);
});

test("a block behind on both axes paints first", () => {
  const far = block(0, 0, 1, 1, 4, 0.5);
  const near = block(2, 2, 1, 1, 4, 0.5);
  assert.deepEqual(painted([near, far]), [far, near]);
});

test("blocks side by side keep the order they came in", () => {
  const left = block(0, 3, 1, 1, 1);
  const right = block(3, 0, 1, 1, 1);
  assert.deepEqual(painted([left, right]), [left, right]);
  assert.deepEqual(painted([right, left]), [right, left]);
});

test("a point inside a top face hits it, one outside misses", () => {
  const top = faces(block(0, 0, 2, 2, 1), 1).top;
  const centre = project(1, 1, 1);
  assert.ok(inside(centre, top));
  assert.ok(!inside(project(5, 1, 5), top));
});

test("bounds hold every corner at full height", () => {
  const [x0, y0, x1, y1] = bounds([block(0, 0, 1, 1, 2)]);
  const top = project(0, 2, 0);
  assert.ok(top[1] >= y0 && top[1] <= y1 && top[0] >= x0 && top[0] <= x1);
});

test("the rise starts at zero, overshoots, and settles at one", () => {
  assert.equal(rise(0), 0);
  assert.equal(rise(1), 1);
  assert.ok(Math.max(...Array.from({ length: 50 }, (_, i) => rise(i / 50))) > 1);
});
