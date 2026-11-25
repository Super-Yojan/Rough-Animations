import { circle, create, createAll, cuboid, cylinder, figure, lag, moveBy, rect, wait, all } from "../../spec.js";
import { defineFigure } from "../types.js";
export const accessDenied = defineFigure({
    id: "access-denied", name: "AccessDenied", title: "Access denied",
    description: "A padlock whose shackle lifts. For permission errors, locked content and sign-in walls.",
    category: "errors", tags: ["lock", "security", "403", "permission", "sign in", "locked"],
    actions: [
        { name: "unlock", label: "Padlock", description: "Request access or sign in." },
    ],
    spec: figure({
        width: 480, height: 360, zoom: 1.5, origin: [240, 280], yaw: 20,
        objects: [
            cuboid(100, 76, 66, { id: "body", baseAt: [0, 0, 0], fill: "amber", action: "unlock" }),
            circle(9, { id: "hole", plane: "front", at: [0, 46, 33.5], fill: "ink", solid: true }),
            rect(6, 20, { id: "slot", plane: "front", at: [0, 32, 33.5], fill: "ink", solid: true }),
            cylinder(8, 46, { id: "legA", baseAt: [-28, 76, 0], fill: "slate", action: "unlock" }),
            cylinder(8, 46, { id: "legB", baseAt: [28, 76, 0], fill: "slate", action: "unlock" }),
            cuboid(64, 12, 16, { id: "bar", baseAt: [0, 122, 0], fill: "slate", action: "unlock" }),
        ],
        timeline: [
            create("body", { duration: 1 }),
            createAll(["hole", "slot"], { duration: 0.4 }),
            lag(0.12, ["legA", "legB", "bar"].map((id) => create(id)), { duration: 0.9 }),
            wait(0.3),
            all(["legA", "legB", "bar"].map((id) => moveBy(id, [0, 16, 0])), { duration: 0.5, ease: "backOut" }),
            wait(0.3),
            all(["legA", "legB", "bar"].map((id) => moveBy(id, [0, -16, 0])), { duration: 0.4, ease: "bounceOut" }),
            wait(0.6),
        ],
    }),
});
