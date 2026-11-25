import { all, create, cuboid, cylinder, figure, lag, moveBy, sphere, star, wait } from "../../spec.js";
import { defineFigure } from "../types.js";

export const success = defineFigure({
  id: "success", name: "Success", title: "Success",
  description: "A wrapped gift with a bow and stars popping. For completed actions, rewards and confirmations.",
  category: "status", tags: ["success", "done", "gift", "celebrate", "confirmation"],
  actions: [{ name: "open", label: "Gift", description: "Open the reward or go to the next step." }],
  spec: figure({
    width: 480, height: 360, zoom: 1.35, origin: [240, 270], yaw: 20,
    objects: [
      cylinder(78, 12, { id: "plate", baseAt: [0, 0, 0], fill: "slate" }),
      cuboid(90, 64, 90, { id: "box", baseAt: [0, 12, 0], fill: "rose", action: "open" }),
      cuboid(96, 14, 96, { id: "lid", baseAt: [0, 76, 0], fill: "rose", action: "open" }),
      cuboid(14, 78, 96, { id: "ribbonA", baseAt: [0, 12, 0], fill: "gold", action: "open" }),
      cuboid(96, 14, 14, { id: "ribbonB", baseAt: [0, 90, 0], fill: "gold", action: "open" }),
      sphere(14, { id: "bow", baseAt: [0, 104, 0], fill: "gold", action: "open" }),
      star(5, 16, 7, { id: "s1", plane: "ground", at: [-70, 150, 20], fill: "gold", solid: true }),
      star(5, 12, 5, { id: "s2", plane: "ground", at: [70, 130, -10], fill: "mint", solid: true }),
      star(5, 10, 4, { id: "s3", plane: "ground", at: [10, 170, 60], fill: "sky", solid: true }),
    ],
    timeline: [
      lag(0.15, ["plate", "box", "lid", "ribbonA", "ribbonB", "bow"].map((id) => create(id)), { duration: 1.5 }),
      all([create("s1"), create("s2"), create("s3")], { duration: 0.5 }),
      all(["s1", "s2", "s3"].map((id) => moveBy(id, [0, 18, 0])), { duration: 0.9, ease: "backOut" }),
      wait(0.6),
    ],
  }),
});
