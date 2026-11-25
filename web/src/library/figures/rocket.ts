import { all, cone, create, cuboid, cylinder, figure, lag, moveBy, sphere, wait } from "../../spec.js";
import { defineFigure } from "../types.js";

const parts = ["body", "nose", "window", "finA", "finB", "finC", "finD"];

export const rocket = defineFigure({
  id: "rocket", name: "Rocket", title: "Rocket",
  description: "A rocket lifting off its pad. For onboarding, launches and getting-started screens.",
  category: "onboarding", tags: ["launch", "start", "onboarding", "rocket", "ship"],
  actions: [
    { name: "launch", label: "Rocket", description: "Start: launch, begin, get started." },
    { name: "fins", label: "Fins", description: "A secondary action, such as customising." },
  ],
  spec: figure({
    width: 480, height: 360, zoom: 1.3, origin: [240, 310], yaw: 25,
    objects: [
      cuboid(120, 14, 120, { id: "pad", baseAt: [0, 0, 0], fill: "slate" }),
      cylinder(26, 110, { id: "body", baseAt: [0, 14, 0], fill: "white", action: "launch" }),
      cone(26, 52, { id: "nose", baseAt: [0, 124, 0], fill: "rose", action: "launch" }),
      sphere(10, { id: "window", baseAt: [14, 86, 14], fill: "sky" }),
      cuboid(8, 46, 34, { id: "finA", baseAt: [36, 14, 0], fill: "rose", action: "fins" }),
      cuboid(34, 46, 8, { id: "finB", baseAt: [0, 14, 36], fill: "rose", action: "fins" }),
      cuboid(8, 46, 34, { id: "finC", baseAt: [-36, 14, 0], fill: "rose", action: "fins" }),
      cuboid(34, 46, 8, { id: "finD", baseAt: [0, 14, -36], fill: "rose", action: "fins" }),
    ],
    timeline: [
      create("pad", { duration: 0.8 }),
      lag(0.15, parts.map((id) => create(id)), { duration: 1.6 }),
      wait(0.3),
      all(parts.map((id) => moveBy(id, [0, 26, 0])), { duration: 1.1, ease: "backOut" }),
      wait(0.8),
    ],
  }),
});
