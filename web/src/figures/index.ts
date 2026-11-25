/**
 * A starter library of figures, all drawn from simple solids. Every clickable part has an
 * `action` name (listed in each figure's doc comment) so a page can wire them to its own UI.
 * Copy one and change it: they are ordinary `FigureSpec` values.
 */
import {
  all, arrow, circle, cone, create, createAll, cuboid, cylinder, fadeIn, figure, fillTo, lag, moveBy,
  moveFrom, ngon, polygon, prism, rect, rotateBy, sphere, star, wait,
  type FigureSpec,
} from "../index.js";

const SIZE = { width: 480, height: 360 } as const;

/**
 * An empty inbox: an open tray with an envelope dropping in.
 * Actions: `compose` (the tray), `open` (the envelope).
 */
export const inbox: FigureSpec = figure({
  ...SIZE, zoom: 1.35, origin: [240, 270], yaw: 20,
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
});

/**
 * Two meshing gears that turn.
 * Actions: `settings` (large gear), `tune` (small gear).
 */
export const settings: FigureSpec = figure({
  ...SIZE, zoom: 1.2, origin: [220, 250], yaw: 0,
  objects: [
    prism(star(12, 56, 45, { minus: circle(16) }), 24, { id: "big", baseAt: [-40, 0, 0], fill: "purple", action: "settings" }),
    prism(star(8, 38, 30, { minus: circle(11) }), 24, { id: "small", baseAt: [66, 0, 4], fill: "mint", action: "tune" }),
  ],
  timeline: [
    lag(0.3, [create("big"), create("small")], { duration: 1.4 }),
    all([rotateBy("big", [0, 70, 0]), rotateBy("small", [0, -105, 0])], { duration: 2.4, ease: "cubicInOut" }),
    wait(0.4),
  ],
});

/**
 * A rocket on its pad.
 * Actions: `launch` (the nose and body), `fins` (the fins).
 */
export const rocket: FigureSpec = figure({
  ...SIZE, zoom: 1.3, origin: [240, 310], yaw: 25,
  objects: [
    cuboid(120, 14, 120, { id: "pad", baseAt: [0, 0, 0], fill: "slate" }),
    cylinder(26, 110, { id: "body", baseAt: [0, 14, 0], fill: "white", action: "launch" }),
    cone(26, 52, { id: "nose", baseAt: [0, 124, 0], fill: "rose", action: "launch" }),
    sphere(10, { id: "window", baseAt: [14, 86, 14], fill: "sky" }),
    cuboid(8, 46, 34, { id: "finA", baseAt: [36, 14, 0], fill: "rose", action: "fins" }),
    cuboid(34, 46, 8, { id: "finB", baseAt: [0, 14, 36], fill: "rose", action: "fins" }),
    cuboid(8, 46, 34, { id: "finC", baseAt: [-36, 14, 0], fill: "rose", action: "fins" }),
    cuboid(34, 46, 8, { id: "finD", baseAt: [0, 14, -36], fill: "rose", action: "fins" }),
    cone(15, 34, { id: "flame", baseAt: [0, -20, 0], rotated: [180, 0, 0], fill: "amber", hidden: true }),
  ],
  timeline: [
    create("pad", { duration: 0.8 }),
    lag(0.15, ["body", "nose", "window", "finA", "finB", "finC", "finD"].map((id) => create(id)), { duration: 1.6 }),
    wait(0.3),
    all(["body", "nose", "window", "finA", "finB", "finC", "finD"].map((id) => moveBy(id, [0, 26, 0])), { duration: 1.1, ease: "backOut" }),
    wait(0.8),
  ],
});

/**
 * A stack of three servers with status lights.
 * Actions: `unit-1`, `unit-2`, `unit-3` (bottom to top).
 */
const unit = (n: number, y: number, light: string) => [
  cuboid(130, 34, 90, { id: `u${n}`, baseAt: [0, y, 0], fill: "slate", action: `unit-${n}` }),
  rect(60, 8, { id: `s${n}`, plane: "front", at: [-22, y + 17, 45.5], fill: "ink", solid: true }),
  circle(5, { id: `l${n}`, plane: "front", at: [40, y + 17, 45.5], fill: light, solid: true }),
];
export const servers: FigureSpec = figure({
  ...SIZE, zoom: 1.25, origin: [240, 270], yaw: 30,
  objects: [...unit(1, 0, "mint"), ...unit(2, 38, "amber"), ...unit(3, 76, "mint")],
  timeline: [
    lag(0.35, ["u1", "u2", "u3"].map((id) => create(id)), { duration: 1.6 }),
    createAll(["s1", "s2", "s3", "l1", "l2", "l3"], { duration: 0.8 }),
    fillTo("l2", "rose", { duration: 0.5 }),
    wait(0.8),
  ],
});

/** Every figure in the library, by name. */
export const figures = { inbox, settings, rocket, servers } as const;
export type FigureName = keyof typeof figures;
