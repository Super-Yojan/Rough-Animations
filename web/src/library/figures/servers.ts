import { circle, create, createAll, cuboid, figure, fillTo, lag, rect, wait } from "../../spec.js";
import { defineFigure } from "../types.js";

const unit = (n: number, y: number, light: string) => [
  cuboid(130, 34, 90, { id: `u${n}`, baseAt: [0, y, 0], fill: "slate", action: `unit-${n}` }),
  rect(60, 8, { id: `s${n}`, plane: "front", at: [-22, y + 17, 45.5], fill: "ink", solid: true }),
  circle(5, { id: `l${n}`, plane: "front", at: [40, y + 17, 45.5], fill: light, solid: true }),
];

export const servers = defineFigure({
  id: "servers", name: "Servers", title: "Servers",
  description: "A stack of three servers with status lights. For infrastructure, hosting and system status.",
  category: "status", tags: ["server", "infrastructure", "hosting", "status", "rack"],
  actions: [
    { name: "unit-1", label: "Bottom server", description: "Open the first server." },
    { name: "unit-2", label: "Middle server", description: "Open the second server (its light is red)." },
    { name: "unit-3", label: "Top server", description: "Open the third server." },
  ],
  spec: figure({
    width: 480, height: 360, zoom: 1.25, origin: [240, 270], yaw: 30,
    objects: [...unit(1, 0, "mint"), ...unit(2, 38, "amber"), ...unit(3, 76, "mint")],
    timeline: [
      lag(0.35, ["u1", "u2", "u3"].map((id) => create(id)), { duration: 1.6 }),
      createAll(["s1", "s2", "s3", "l1", "l2", "l3"], { duration: 0.8 }),
      fillTo("l2", "rose", { duration: 0.5 }),
      wait(0.8),
    ],
  }),
});
