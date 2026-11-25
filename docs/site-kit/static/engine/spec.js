/**
 * Typed scene descriptions and small builder functions. They produce the JSON the engine reads
 * (see `src/spec.rs`), so a figure is checked by the compiler instead of by a runtime error.
 * Angles are in degrees.
 */
/** Identity function that gives you completion and type checking for a figure literal. */
export const figure = (spec) => spec;
// Snake-case the camelCase style keys the engine expects.
const snake = (k) => k.replace(/[A-Z]/g, (c) => "_" + c.toLowerCase());
function wire(value) {
    if (Array.isArray(value))
        return value.map(wire);
    if (value && typeof value === "object") {
        return Object.fromEntries(Object.entries(value)
            .filter(([, v]) => v !== undefined)
            .map(([k, v]) => [snake(k), wire(v)]));
    }
    return value;
}
/** The JSON the engine reads. */
export const toJSON = (spec) => JSON.stringify(wire(spec));
// ---- objects -------------------------------------------------------------------------------
const obj = (kind, fields, s = {}) => ({ kind, ...fields, ...s });
export const cuboid = (w, h, d, s) => obj("cuboid", { size: [w, h, d] }, s);
export const cube = (size, s) => obj("cube", { size }, s);
export const cylinder = (radius, height, s) => obj("cylinder", { radius, height }, s);
export const cone = (radius, height, s) => obj("cone", { radius, height }, s);
export const sphere = (radius, s) => obj("sphere", { radius }, s);
export const circle = (radius, s) => obj("circle", { radius }, s);
export const ellipse = (rx, ry, s) => obj("ellipse", { rx, ry }, s);
export const rect = (width, height, s) => obj("rect", { width, height }, s);
export const ngon = (sides, radius, s) => obj("ngon", { sides, radius }, s);
export const star = (points, outer, inner, s) => obj("star", { points, outer, inner }, s);
export const polygon = (points, s) => obj("polygon", { points }, s);
/** Extrude a flat shape upward into a solid. Combine shapes with `minus` etc. before extruding. */
export const prism = (shape, height, s) => obj("prism", { shape, height }, s);
export const line = (from, to, s) => obj("line", { from, to }, s);
export const arrow = (from, to, s) => obj("arrow", { from, to }, s);
export const text = (text, s) => obj("text", { text }, s);
export const label = (text, s) => obj("label", { text }, s);
export const title = (text, s) => obj("title", { text }, s);
const anim = (op, fields, t = {}) => ({ op, ...fields, ...t });
export const create = (target, t) => anim("create", { target }, t);
export const write = (target, t) => anim("write", { target }, t);
export const fadeIn = (target, t) => anim("fade_in", { target }, t);
export const uncreate = (target, t) => anim("uncreate", { target }, t);
export const fadeOut = (target, t) => anim("fade_out", { target }, t);
export const moveTo = (target, to, t) => anim("move_to", { target, to }, t);
export const moveBy = (target, by, t) => anim("move_by", { target, by }, t);
export const moveFrom = (target, by, t) => anim("move_from", { target, by }, t);
export const rotateTo = (target, to, t) => anim("rotate_to", { target, to }, t);
export const rotateBy = (target, by, t) => anim("rotate_by", { target, by }, t);
export const scaleTo = (target, to, t) => anim("scale_to", { target, to }, t);
export const opacityTo = (target, to, t) => anim("opacity_to", { target, to }, t);
export const fillTo = (target, to, t) => anim("fill_to", { target, to }, t);
export const strokeTo = (target, to, t) => anim("stroke_to", { target, to }, t);
export const morph = (target, shape, t) => anim("morph", { target, shape }, t);
export const wait = (seconds) => anim("wait", { seconds });
export const all = (anims, t) => anim("all", { anims }, t);
export const seq = (anims, t) => anim("seq", { anims }, t);
/** Start each animation `gap` seconds after the one before it. */
export const lag = (gap, anims, t) => anim("lag", { seconds: gap, anims }, t);
export const createAll = (targets, t) => anim("create_all", { targets }, t);
export const writeAll = (targets, t) => anim("write_all", { targets }, t);
export const clear = (targets, t) => anim("clear", { targets }, t);
