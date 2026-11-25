import { all, circle, create, figure, lag, prism, rotateBy, star, wait } from "../../spec.js";
import { defineFigure } from "../types.js";

export const settings = defineFigure({
  id: "settings", name: "Settings", title: "Settings",
  description: "Two gears that turn. For settings pages, preferences and configuration screens.",
  category: "product", tags: ["settings", "gear", "preferences", "configuration"],
  actions: [
    { name: "settings", label: "Large gear", description: "Open the main settings." },
    { name: "tune", label: "Small gear", description: "Open the fine-tuning options." },
  ],
  spec: figure({
    width: 480, height: 360, zoom: 1.2, origin: [220, 250],
    objects: [
      prism(star(12, 56, 45, { minus: circle(16) }), 24, { id: "big", baseAt: [-40, 0, 0], fill: "purple", action: "settings" }),
      prism(star(8, 38, 30, { minus: circle(11) }), 24, { id: "small", baseAt: [66, 0, 4], fill: "mint", action: "tune" }),
    ],
    timeline: [
      lag(0.3, [create("big"), create("small")], { duration: 1.4 }),
      all([rotateBy("big", [0, 70, 0]), rotateBy("small", [0, -105, 0])], { duration: 2.4, ease: "cubicInOut" }),
      wait(0.4),
    ],
  }),
});
