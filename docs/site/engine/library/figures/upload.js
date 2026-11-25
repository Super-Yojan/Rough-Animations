import { all, arrow, create, cube, cuboid, figure, lag, moveBy, wait } from "../../spec.js";
import { defineFigure } from "../types.js";
export const upload = defineFigure({
    id: "upload", name: "Upload", title: "Upload",
    description: "A file lifting off a tray with an arrow. For upload areas, imports and drop zones.",
    category: "empty-states", tags: ["upload", "file", "import", "drop", "arrow"],
    actions: [
        { name: "upload", label: "Tray and file", description: "Choose or drop a file." },
    ],
    spec: figure({
        width: 480, height: 360, zoom: 1.35, origin: [240, 280], yaw: 25,
        objects: [
            cuboid(150, 14, 110, { id: "tray", baseAt: [0, 0, 0], fill: "slate", action: "upload" }),
            cuboid(100, 6, 70, { id: "edge", baseAt: [0, 14, 0], fill: "white", action: "upload" }),
            cube(46, { id: "file", baseAt: [0, 20, 0], fill: "sky", action: "upload" }),
            arrow([0, 90, 0], [0, 140, 0], { id: "up", color: "indigo", strokeWidth: 3, action: "upload" }),
        ],
        timeline: [
            lag(0.2, ["tray", "edge", "file"].map((id) => create(id)), { duration: 1.2 }),
            create("up", { duration: 0.5 }),
            all([moveBy("file", [0, 46, 0]), moveBy("up", [0, 22, 0])], { duration: 0.9, ease: "backOut" }),
            wait(0.6),
        ],
    }),
});
