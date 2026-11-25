/**
 * Typed scene descriptions and small builder functions. They produce the JSON the engine reads
 * (see `src/spec.rs`), so a figure is checked by the compiler instead of by a runtime error.
 * Angles are in degrees.
 */

export type Vec3 = [number, number, number];
export type ThemeName = "light" | "dark" | "chalkboard" | "blueprint" | "sepia" | "terminal";
export type EaseName =
  | "linear" | "quadIn" | "quadOut" | "quadInOut" | "cubicIn" | "cubicOut"
  | "cubicInOut" | "sineInOut" | "backOut" | "bounceOut";

/** A colour: a name (`"purple"`, `"mint"`, `"rose"`, ...) or `"#rrggbb"`. */
export type Color = string;

export type Kind =
  | "cuboid" | "cube" | "cylinder" | "cone" | "sphere"
  | "circle" | "ellipse" | "rect" | "ngon" | "star" | "polygon"
  | "prism" | "line" | "arrow" | "text" | "label" | "title";

/** Options every object accepts. */
export interface Style {
  /** Name that animations and `action` events refer to. */
  id?: string;
  at?: Vec3;
  /** Stand the object on this point instead of centring it. */
  baseAt?: Vec3;
  rotated?: Vec3;
  scaled?: number | Vec3;
  fill?: Color;
  stroke?: Color;
  /** Fill and stroke together. */
  color?: Color;
  opacity?: number;
  strokeWidth?: number;
  /** Lay a flat shape or text on a wall of the scene. */
  plane?: "front" | "side" | "ground";
  shift?: [number, number];
  solid?: boolean;
  hatched?: boolean;
  fontSize?: number;
  tracking?: number;
  animate?: boolean;
  handwritten?: boolean;
  layer?: number;
  hidden?: boolean;
  /** Makes the part interactive: it highlights on hover and reports this name when clicked. */
  action?: string;
  union?: ObjectSpec;
  minus?: ObjectSpec;
  intersect?: ObjectSpec;
  xor?: ObjectSpec;
}

export interface ObjectSpec extends Style {
  kind: Kind;
  size?: number | Vec3;
  radius?: number;
  height?: number;
  width?: number;
  rx?: number;
  ry?: number;
  sides?: number;
  points?: number | [number, number][];
  outer?: number;
  inner?: number;
  from?: Vec3;
  to?: Vec3;
  text?: string;
  shape?: ObjectSpec;
}

export type Op =
  | "create" | "write" | "fade_in" | "uncreate" | "fade_out"
  | "move_to" | "move_by" | "move_from" | "rotate_to" | "rotate_by"
  | "scale_to" | "opacity_to" | "fill_to" | "stroke_to" | "morph"
  | "wait" | "all" | "seq" | "lag" | "create_all" | "write_all" | "clear";

export interface AnimSpec {
  op: Op;
  target?: string;
  targets?: string[];
  to?: number | Vec3 | Color;
  by?: Vec3;
  duration?: number;
  delay?: number;
  ease?: EaseName;
  seconds?: number;
  anims?: AnimSpec[];
  shape?: ObjectSpec;
}

export interface FigureSpec {
  width: number;
  height: number;
  fps?: number;
  zoom?: number;
  origin?: [number, number];
  /** Starting turn about the vertical axis, in degrees. */
  yaw?: number;
  theme?: ThemeName;
  /** A colour, or `"transparent"` to let the page behind the canvas show. */
  background?: Color | "transparent";
  look?: "sketch" | "clean";
  roughness?: number;
  /** Redraw strokes this many times a second. */
  wobble?: number;
  objects: ObjectSpec[];
  timeline?: AnimSpec[];
}

/** Identity function that gives you completion and type checking for a figure literal. */
export const figure = (spec: FigureSpec): FigureSpec => spec;

// Snake-case the camelCase style keys the engine expects.
const snake = (k: string) => k.replace(/[A-Z]/g, (c) => "_" + c.toLowerCase());
function wire(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(wire);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .filter(([, v]) => v !== undefined)
        .map(([k, v]) => [snake(k), wire(v)]),
    );
  }
  return value;
}

/** The JSON the engine reads. */
export const toJSON = (spec: FigureSpec): string => JSON.stringify(wire(spec));

// ---- objects -------------------------------------------------------------------------------

const obj = (kind: Kind, fields: Omit<ObjectSpec, "kind">, s: Style = {}): ObjectSpec => ({ kind, ...fields, ...s });

export const cuboid = (w: number, h: number, d: number, s?: Style) => obj("cuboid", { size: [w, h, d] }, s);
export const cube = (size: number, s?: Style) => obj("cube", { size }, s);
export const cylinder = (radius: number, height: number, s?: Style) => obj("cylinder", { radius, height }, s);
export const cone = (radius: number, height: number, s?: Style) => obj("cone", { radius, height }, s);
export const sphere = (radius: number, s?: Style) => obj("sphere", { radius }, s);
export const circle = (radius: number, s?: Style) => obj("circle", { radius }, s);
export const ellipse = (rx: number, ry: number, s?: Style) => obj("ellipse", { rx, ry }, s);
export const rect = (width: number, height: number, s?: Style) => obj("rect", { width, height }, s);
export const ngon = (sides: number, radius: number, s?: Style) => obj("ngon", { sides, radius }, s);
export const star = (points: number, outer: number, inner: number, s?: Style) => obj("star", { points, outer, inner }, s);
export const polygon = (points: [number, number][], s?: Style) => obj("polygon", { points }, s);
/** Extrude a flat shape upward into a solid. Combine shapes with `minus` etc. before extruding. */
export const prism = (shape: ObjectSpec, height: number, s?: Style) => obj("prism", { shape, height }, s);
export const line = (from: Vec3, to: Vec3, s?: Style) => obj("line", { from, to }, s);
export const arrow = (from: Vec3, to: Vec3, s?: Style) => obj("arrow", { from, to }, s);
export const text = (text: string, s?: Style) => obj("text", { text }, s);
export const label = (text: string, s?: Style) => obj("label", { text }, s);
export const title = (text: string, s?: Style) => obj("title", { text }, s);

// ---- animations ----------------------------------------------------------------------------

type Tweak = Pick<AnimSpec, "duration" | "delay" | "ease">;
const anim = (op: Op, fields: Omit<AnimSpec, "op">, t: Tweak = {}): AnimSpec => ({ op, ...fields, ...t });

export const create = (target: string, t?: Tweak) => anim("create", { target }, t);
export const write = (target: string, t?: Tweak) => anim("write", { target }, t);
export const fadeIn = (target: string, t?: Tweak) => anim("fade_in", { target }, t);
export const uncreate = (target: string, t?: Tweak) => anim("uncreate", { target }, t);
export const fadeOut = (target: string, t?: Tweak) => anim("fade_out", { target }, t);
export const moveTo = (target: string, to: Vec3, t?: Tweak) => anim("move_to", { target, to }, t);
export const moveBy = (target: string, by: Vec3, t?: Tweak) => anim("move_by", { target, by }, t);
export const moveFrom = (target: string, by: Vec3, t?: Tweak) => anim("move_from", { target, by }, t);
export const rotateTo = (target: string, to: Vec3, t?: Tweak) => anim("rotate_to", { target, to }, t);
export const rotateBy = (target: string, by: Vec3, t?: Tweak) => anim("rotate_by", { target, by }, t);
export const scaleTo = (target: string, to: number | Vec3, t?: Tweak) => anim("scale_to", { target, to }, t);
export const opacityTo = (target: string, to: number, t?: Tweak) => anim("opacity_to", { target, to }, t);
export const fillTo = (target: string, to: Color, t?: Tweak) => anim("fill_to", { target, to }, t);
export const strokeTo = (target: string, to: Color, t?: Tweak) => anim("stroke_to", { target, to }, t);
export const morph = (target: string, shape: ObjectSpec, t?: Tweak) => anim("morph", { target, shape }, t);
export const wait = (seconds: number) => anim("wait", { seconds });
export const all = (anims: AnimSpec[], t?: Tweak) => anim("all", { anims }, t);
export const seq = (anims: AnimSpec[], t?: Tweak) => anim("seq", { anims }, t);
/** Start each animation `gap` seconds after the one before it. */
export const lag = (gap: number, anims: AnimSpec[], t?: Tweak) => anim("lag", { seconds: gap, anims }, t);
export const createAll = (targets: string[], t?: Tweak) => anim("create_all", { targets }, t);
export const writeAll = (targets: string[], t?: Tweak) => anim("write_all", { targets }, t);
export const clear = (targets: string[], t?: Tweak) => anim("clear", { targets }, t);
