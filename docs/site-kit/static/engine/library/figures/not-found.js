import { create, cuboid, figure, lag, rotateBy, text, wait, all, moveBy } from "../../spec.js";
import { defineFigure } from "../types.js";
export const notFound = defineFigure({
    id: "not-found", name: "NotFound", title: "Not found (404)",
    description: "Three blocks spelling 404, the last one knocked over. For missing pages and empty search results.",
    category: "errors", tags: ["404", "error", "missing", "page not found", "search"],
    actions: [
        { name: "home", label: "First 4", description: "Go back home." },
        { name: "search", label: "Zero", description: "Search for what was lost." },
    ],
    spec: figure({
        width: 480, height: 360, zoom: 1.0, origin: [240, 250], yaw: 15,
        objects: [
            cuboid(70, 80, 70, { id: "a", baseAt: [-100, 0, 0], fill: "purple", action: "home" }),
            text("4", { id: "ta", plane: "front", at: [-100, 40, 35.5], fontSize: 60, color: "ink" }),
            cuboid(70, 80, 70, { id: "b", baseAt: [0, 0, 0], fill: "mint", action: "search" }),
            text("0", { id: "tb", plane: "front", at: [0, 40, 35.5], fontSize: 60, color: "ink" }),
            cuboid(70, 80, 70, { id: "c", baseAt: [100, 0, 0], fill: "rose" }),
            text("4", { id: "tc", plane: "front", at: [100, 40, 35.5], fontSize: 60, color: "ink" }),
        ],
        timeline: [
            lag(0.3, ["a", "b", "c"].map((id) => create(id)), { duration: 1.5 }),
            lag(0.2, ["ta", "tb", "tc"].map((id) => create(id)), { duration: 1 }),
            all([rotateBy("c", [0, 0, -12]), rotateBy("tc", [0, 0, -12]), moveBy("c", [10, 0, 0]), moveBy("tc", [10, 0, 0])], { duration: 0.7, ease: "bounceOut" }),
            wait(0.6),
        ],
    }),
});
