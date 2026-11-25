import { all, create, cylinder, figure, lag, moveBy, sphere, wait } from "../../spec.js";
import { defineFigure } from "../types.js";
export const notifications = defineFigure({
    id: "notifications", name: "Notifications", title: "Notifications",
    description: "A bell with an unread badge. For notification settings, alerts and reminders.",
    category: "status", tags: ["bell", "notifications", "alerts", "unread", "reminders"],
    actions: [
        { name: "open", label: "Bell", description: "Open the notifications." },
        { name: "dismiss", label: "Badge", description: "Mark as read." },
    ],
    spec: figure({
        width: 480, height: 360, zoom: 1.5, origin: [240, 290], yaw: 15,
        objects: [
            cylinder(66, 10, { id: "base", baseAt: [0, 0, 0], fill: "slate" }),
            cylinder(58, 14, { id: "rim", baseAt: [0, 24, 0], fill: "gold", action: "open" }),
            sphere(48, { id: "bell", baseAt: [0, 36, 0], fill: "gold", action: "open" }),
            sphere(9, { id: "top", baseAt: [0, 132, 0], fill: "amber", action: "open" }),
            sphere(13, { id: "clapper", baseAt: [0, 10, 0], fill: "amber", action: "open" }),
            sphere(17, { id: "badge", baseAt: [44, 92, 34], fill: "rose", action: "dismiss" }),
        ],
        timeline: [
            lag(0.15, ["base", "rim", "bell", "top", "clapper"].map((id) => create(id)), { duration: 1.3 }),
            create("badge", { duration: 0.4 }),
            all(["rim", "bell", "top", "badge"].map((id) => moveBy(id, [8, 0, 0])), { duration: 0.16 }),
            all(["rim", "bell", "top", "badge"].map((id) => moveBy(id, [-16, 0, 0])), { duration: 0.2 }),
            all(["rim", "bell", "top", "badge"].map((id) => moveBy(id, [8, 0, 0])), { duration: 0.16 }),
            wait(0.6),
        ],
    }),
});
