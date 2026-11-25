import test from "node:test";
import assert from "node:assert/strict";
import { cuboid, prism, rect, circle, create, lag, toJSON, figure } from "../dist/spec.js";

test("style keys are sent to the engine in snake_case, undefined ones are dropped", () => {
  const json = JSON.parse(toJSON(figure({
    width: 100, height: 80,
    objects: [cuboid(1, 2, 3, { id: "a", baseAt: [0, 0, 0], strokeWidth: 3, fill: undefined })],
  })));
  assert.deepEqual(json.objects[0], { kind: "cuboid", size: [1, 2, 3], id: "a", base_at: [0, 0, 0], stroke_width: 3 });
});

test("nested shapes and animations are converted too", () => {
  const spec = figure({
    width: 10, height: 10,
    objects: [prism(rect(10, 10, { minus: circle(3) }), 4, { id: "p", fontSize: 12 })],
    timeline: [lag(0.2, [create("p")], { duration: 1 })],
  });
  const json = JSON.parse(toJSON(spec));
  assert.equal(json.objects[0].shape.minus.kind, "circle");
  assert.equal(json.objects[0].font_size, 12);
  assert.deepEqual(json.timeline[0], { op: "lag", seconds: 0.2, anims: [{ op: "create", target: "p" }], duration: 1 });
});
