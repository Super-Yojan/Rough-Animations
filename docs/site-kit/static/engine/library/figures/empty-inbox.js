import { all, create, cuboid, figure, lag, moveBy, polygon, wait } from "../../spec.js";
import { defineFigure } from "../types.js";
export const emptyInbox = defineFigure({
    id: "empty-inbox", name: "EmptyInbox", title: "Empty inbox",
    description: "An open tray with an envelope dropping in. For inboxes, notifications and message lists with nothing in them yet.",
    category: "empty-states", tags: ["inbox", "mail", "messages", "notifications", "empty"],
    actions: [
        { name: "compose", label: "Tray", description: "Start something new (compose, add, create)." },
        { name: "open", label: "Envelope", description: "Open the thing that arrived." },
    ],
    spec: figure({
        width: 480, height: 360, zoom: 1.35, origin: [240, 270], yaw: 20,
        objects: [
            cuboid(150, 10, 100, { id: "floor", baseAt: [0, 0, 0], fill: "slate", action: "compose" }),
            cuboid(150, 36, 8, { id: "back", baseAt: [0, 10, -46], fill: "slate", action: "compose" }),
            cuboid(8, 36, 84, { id: "left", baseAt: [-71, 10, 0], fill: "slate", action: "compose" }),
            cuboid(150, 16, 8, { id: "front", baseAt: [0, 10, 46], fill: "slate", action: "compose" }),
            cuboid(8, 16, 84, { id: "right", baseAt: [71, 10, 0], fill: "slate", action: "compose" }),
            cuboid(72, 6, 48, { id: "paper", baseAt: [0, 64, 0], fill: "white", action: "open" }),
            polygon([[-36, 24], [36, 24], [0, -4]], { id: "flap", plane: "ground", at: [0, 70.5, 0], fill: "amber", action: "open" }),
        ],
        timeline: [
            lag(0.18, ["floor", "back", "left", "front", "right"].map((id) => create(id)), { duration: 1.4 }),
            all([create("paper"), create("flap")], { duration: 0.7 }),
            all([moveBy("paper", [0, -34, 0]), moveBy("flap", [0, -34, 0])], { duration: 0.9, ease: "bounceOut" }),
            wait(0.6),
        ],
    }),
});
