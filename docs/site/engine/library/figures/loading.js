import { create, cube, figure, lag, moveBy, seq, wait } from "../../spec.js";
import { defineFigure } from "../types.js";
const hop = (id) => seq([moveBy(id, [0, 34, 0], { ease: "quadOut", duration: 0.3 }), moveBy(id, [0, -34, 0], { ease: "bounceOut", duration: 0.45 })]);
export const loading = defineFigure({
    id: "loading", name: "Loading", title: "Loading",
    description: "Three cubes hopping in turn. Loop it for a spinner with some character.",
    category: "loaders", tags: ["loading", "spinner", "wait", "progress"],
    actions: [],
    spec: figure({
        width: 480, height: 360, zoom: 1.3, origin: [240, 250], yaw: 20,
        objects: [
            cube(60, { id: "a", baseAt: [-80, 0, 0], fill: "purple" }),
            cube(60, { id: "b", baseAt: [0, 0, 0], fill: "mint" }),
            cube(60, { id: "c", baseAt: [80, 0, 0], fill: "amber" }),
        ],
        timeline: [
            lag(0.2, ["a", "b", "c"].map((id) => create(id)), { duration: 1.0 }),
            lag(0.18, ["a", "b", "c"].map(hop)),
            lag(0.18, ["a", "b", "c"].map(hop)),
            wait(0.3),
        ],
    }),
});
