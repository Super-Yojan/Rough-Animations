import { arrow, create, cuboid, figure, lag, moveFrom, wait } from "../../spec.js";
import { defineFigure } from "../types.js";

const heights = [30, 52, 40, 76, 100];
const colors = ["sky", "mint", "amber", "purple", "rose"];

export const analytics = defineFigure({
  id: "analytics", name: "Analytics", title: "Analytics",
  description: "Five bars growing with a trend arrow. For dashboards, reports and growth screens.",
  category: "product", tags: ["chart", "bars", "analytics", "dashboard", "growth", "report"],
  actions: [
    { name: "report", label: "Bars", description: "Open the report." },
    { name: "trend", label: "Arrow", description: "Open the trend details." },
  ],
  spec: figure({
    width: 480, height: 360, zoom: 1.15, origin: [240, 285], yaw: 28,
    objects: [
      cuboid(250, 12, 80, { id: "base", baseAt: [0, 0, 0], fill: "slate" }),
      ...heights.map((h, i) => cuboid(36, h, 40, { id: `bar${i}`, baseAt: [-96 + i * 48, 12, 0], fill: colors[i]!, action: "report" })),
      arrow([-112, 110, 0], [116, 220, 0], { id: "trend", color: "indigo", strokeWidth: 3, action: "trend" }),
    ],
    timeline: [
      create("base", { duration: 0.6 }),
      lag(0.15, heights.map((_, i) => create(`bar${i}`)), { duration: 1.4 }),
      create("trend", { duration: 0.7 }),
      wait(0.8),
    ],
  }),
});
