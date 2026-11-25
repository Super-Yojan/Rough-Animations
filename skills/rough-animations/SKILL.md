---
name: rough-animations
description: Make hand-drawn style 3D and 2D animations, diagrams and explainer videos in Rust with the rough-animations library, rendered to mp4, gif or png. Covers isometric solids, flat shapes with boolean operations, morphing, text, handwritten LaTeX, and how to check the result without watching it. Use when asked to animate, draw, sketch, illustrate or make a video or diagram with this library, to build an avatar or scene with it, or when editing its examples.
---

# rough-animations

Script a `Stage` in Rust, then `render` it to mp4, gif or png. This skill is how to work with the
library and how to check your output; the full API is at the bottom.

## Workflow

1. **Where the script goes.** In this repo, write `examples/<name>.rs` and run it with
   `cargo run --release --example <name>` (always `--release`; debug builds are far slower).
   In another project, add the dependency and write `src/main.rs`:

   ```toml
   [dependencies]
   rough-animations = { git = "https://github.com/Super-Yojan/rough-animations" }
   ```

2. **Start from a template.** The library's `examples/` folder (in its repo, at
   https://github.com/Super-Yojan/rough-animations/tree/main/examples) has runnable scripts:
   `examples/hello.rs` is the smallest, `examples/avatar.rs` is a fuller scene (a 3D character that
   draws itself, blinks and smiles), `examples/intro.rs` shows multi-scene layout, and
   `examples/text.rs` shows labels and LaTeX.

3. **Draw a still first.** Layout mistakes are far cheaper to find in one picture than in a
   video. `s.render("target/check.png")` draws the finished scene with no ffmpeg needed. Open the
   image and look at it before doing anything else.

4. **Then check the motion.** You cannot watch a video, so sample it:

   The helper scripts are in the `scripts/` folder next to this file (for example
   `~/.claude/skills/rough-animations/scripts`). Set `S` to that folder:

   ```sh
   S=~/.claude/skills/rough-animations/scripts
   $S/contact-sheet.sh target/out.mp4 1.5 4 target/sheet.png   # every 1.5 s, 4 per row
   $S/frame.sh target/out.mp4 3.2 target/at3.png               # one frame at 3.2 s
   $S/frame.sh target/out.mp4 -0.5 target/end.png              # a negative time counts from the end
   ```

   Open the PNGs. On a contact sheet, look for: things overlapping or clipped at the edges, text
   colliding with shapes, an object that never appears, and a last frame that is unfinished.

5. **Fix what you saw and render again.** Change the script, rerun, look again. Do not render two
   things to the same output file at the same time, because the file ends up half-written.

## Design guidance

- **Lay out in pixels.** `s.put(obj, x, y)` places something on a screen pixel, `s.heading("Title",
  "caption")` adds a header, and `s.tag(id, "name", Side::Above)` labels an object. Use
  `Stage::zoom` to make a scene fill the canvas and `Stage::origin` to move it.
- **Stack with `base_at`.** Solids are centred on their position, so `base_at(p)` (a point on the
  floor or on top of another object) saves you the half-height arithmetic.
- **Put faces on flat sides.** To draw on a box, lay flat shapes on its visible face with
  `.on(Plane::Front)` (the face at `z = +half-size`) and nudge them a hair in front. This keeps
  the depth ordering exact. `examples/avatar.rs` does exactly this.
- **Things made of parts.** Put the parts in a `Group` (`s.add_to(&mut rover, ...)`), then move, turn
  and fade it as one: `rover.move_by(..)`, `rover.rotate_by((0, deg(90), 0))`, `rover.create()`.
  Do not call `move_to` on each part, which stacks them all on one point. `examples/rover.rs` is a
  worked example.
- **One idea per scene.** Build it, `write_all` / `create_all` its parts, hold for a moment with
  `s.wait(..)`, then `clear(&ids)` before the next scene. Use `lag(0.3, [..])` to stagger, and
  `all([..])` to combine.
- **Text.** It is crisp and steady by default so it stays readable. Use `title(..)` (or
  `.animate()`) for a lively wobble on a *big* title only. Use `.handwritten()` to make a formula
  or label look hand-written. Billboard text is always drawn on top of solids.
- **Dark pages and other looks.** Use a theme: `Stage::new(w, h).theme("dark")` (also `"chalkboard"`,
  `"blueprint"`, `"sepia"`, `"terminal"`, or your own `Theme::custom(page, ink)`). It sets the page and the
  default colours of lines, text, titles and captions together. Setting only `.background(..)` to a dark
  colour leaves the dark default ink on it, so everything disappears.
- **Colour.** Names (`"purple"`, `"mint"`, ...) or hex. For flat face parts use `.solid()` so they
  fill cleanly instead of hatching.

## When something goes wrong

| You see | Cause and fix |
|---|---|
| panic `unknown colour "..."` | Not a palette name. Use one from the list below or `"#rrggbb"`. |
| panic `on(): only flat shapes ...` | `.on(plane)` is for flat shapes and text, not solids. |
| panic `boolean operations need flat shapes` | `union/minus/...` work on `circle`, `rect`, ... Extrude the result with `prism`. |
| `this Stage has already played` | A stage plays once. Build a fresh one per video (a `.png` may follow a video). |
| `ffmpeg not found` | Install ffmpeg for mp4/webm/gif. PNG output works without it. |
| `LaTeX error: Undefined control sequence` | A macro is misspelled. `math` needs `pdflatex` plus `pdftocairo` or `mutool`. |
| object drawn in the wrong depth order | Solids that pass through each other are sorted by their centres. Keep them apart. |
| a title lands in the middle of the scene | `s.add(title(..))` goes top-centre only when you give no `.at(..)`. |

## Limits

The camera is isometric only (a fixed `zoom` and `origin`, no perspective or camera moves). Booleans
are 2D, with no 3D booleans. Text has no kerning, and upright labels cannot be hidden behind solids.

## API reference

The library's cheat sheet (`docs/cheatsheet.md` in its repo), verbatim.

<!-- cheatsheet:start -->
# rough-animations

Hand-drawn style 3D/2D animations in Rust, like manim. Script a `Stage`, `render` to mp4/gif/png.

```rust,no_run
use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600);                                   // sketch look, iso camera
    let tower = s.add(cuboid(80, 110, 80).base_at((0, 0, 0)).fill("purple"));
    let note = s.add(text("compute").at((0, 150, 0)));
    s.play(create(tower));                                              // outline traced, then filled
    s.play(write(note));
    s.play(all([move_by(tower, (100, 0, 0)), fill_to(tower, "rose")]).duration(1.5));
    s.wait(0.5);
    s.render("out.mp4")                                                 // .mp4 .webm .gif, or .png = last frame
}
```

## World
x runs to the lower right, z to the lower left, **y is up**. `(0, 0, 0)` is at the screen point set
by `Stage::origin(x, y)` (default: centre, 60% down). Sizes, positions, times and angles can be ints or
floats; only counts (`ngon` sides, `star` points) must be whole numbers.
Positions are `(x, y, z)` tuples. Solids are centred on their position; **`.base_at(p)` stands them
on `p`** instead. Rotations are radians: `deg(90)`.

## Shapes (all return `Object`, chain builders on them)
| 3D solids | 2D flat shapes (lie on the floor; `.on(Plane::Front)` for a wall) | other |
|---|---|---|
| `cuboid(w,h,d)` `cube(s)` | `circle(r)` `ellipse(rx,ry)` `rect(w,h)` | `line(a,b)` `arrow(a,b)` `label("..")` (steady caption) `title("..")` (big, lively) |
| `cylinder(r,h)` `cone(r,h)` `sphere(r)` | `ngon(sides,r)` `star(n,outer,inner)` `polygon([(x,y),..])` | `text("..")` `math(r"\frac12")?` `latex("..")?` |
| `prism(flat_shape, h)` extrudes a flat shape | booleans: `.union(b)` `.minus(b)` `.intersect(b)` `.xor(b)` | |

Builders: `.at(p)` `.base_at(p)` `.fill(c)` `.stroke(c)` `.color(c)` (fill+stroke) `.opacity(0..1)`
`.stroke_width(w)` `.rotated(r)` `.scaled(s)` `.layer(n)` `.hidden()`, flat shapes: `.shift(dx,dy)` `.on(plane)`,
text: `.font_size(n)` `.tracking(em)` (letter spacing, default 0.06) `.animate()` `.steady()` `.handwritten()` (see below). Colours: `"purple"` `"mint"` `"amber"` `"sky"` `"rose"` `"gold"` `"slate"` `"ink"` `"indigo"` `"stone"`
`"red"` `"green"` `"blue"` `"orange"` `"yellow"` `"pink"` `"teal"` `"brown"` `"gray"` `"black"` `"white"`,
`"#a882ff"`, or `(r, g, b)`. A wrong name panics and lists the valid ones.

Combine shapes **before** placing them: `prism(rect(80,80).minus(circle(25)), 40).base_at(p)`.

## Animations (each returns `Anim`; `id = s.add(..)`)
Appear: `create(id)` `write(id)` (same; text letter by letter) `fade_in(id)`. Disappear: `uncreate(id)` `fade_out(id)`.
Move: `move_to(id,p)` `move_by(id,d)` `move_from(id,offset)` (settles onto the current spot).
Change: `rotate_to/by(id,r)` `scale_to(id,s)` `opacity_to(id,a)` `fill_to(id,c)` `stroke_to(id,c)`.
Shape: `morph(id, star(5,70,30))` into another flat shape or prism; `transform_into(id, other_id)` also takes its
position and colours. Time: `wait(secs)`.
Tweak: `.duration(secs)` (total length) `.ease(Easing::BackOut)` `.delay(secs)` `.then(b)` `.with(b)`.
Group (no boxing): `all([a,b])` together, `seq([a,b])` in order, `lag(0.3, [a,b,c])` staggered.
Lists of ids: `write_all(&ids)` `create_all(&ids)` (staggered), `clear(&ids)` (erase together, 0.7 s).

## Stage
`Stage::new(w,h)` then optional `.fps(n)` `.background(c)` `.origin(x,y)` `.zoom(f)` (bigger scene, same line
widths and text) `.wobble(fps)` (boiling lines) `.sketch(Sketch::clean())`. `s.add(obj) -> Id`, `s.play(anim)`,
`s.wait(secs)`, `s.render(path)`. **`s.pixel(x, y)`** is the floor point under a screen pixel, so
`text("Title").at(s.pixel(640, 80))` puts text exactly there (call it after `origin`/`zoom`).
**For slides and documents:** `Stage::new(..).transparent()` leaves the page see-through (shapes still hide the lines
behind them), and `s.still_cropped("fig.png", 16)` saves the final frame trimmed to the drawing plus 16 px. PNG and GIF
keep the transparency; `.mp4` and `.webm` cannot, and return an error instead of turning black. `"transparent"` and
`"#rrggbbaa"` also work as colours. Ink stays the theme's, so for a dark slide use `.theme("dark").transparent()`.

## Themes
`Stage::new(w, h).theme("dark")` sets the page colour and the default colours of lines, text, titles and captions
together. Built in: `"light"` (the default) `"dark"` `"chalkboard"` `"blueprint"` `"sepia"` `"terminal"`. Some bring
their own strokes (blueprint and terminal are drawn straight, chalkboard rougher); a later `.sketch(..)` overrides that.
Make your own with `Theme::custom("#06283d", "#e0f2ff").title("#7dd3fc")` (page, then ink; also `.caption(c)` `.ink(c)`
`.background(c)` `.sketch(s)`) and pass it to `.theme(..)`. Anything you colour yourself (`.fill`, `.stroke`, `.color`)
is left alone, and the theme reaches objects added before it too.

## Groups
Several parts that move together (a rover, a character): `let mut car = Group::new();` then
`s.add_to(&mut car, cuboid(..))` for each part (it returns the part's id, so you can still animate one part alone).
Animate the whole thing as one: `car.move_by((x,y,z))`, `car.move_to(p)` (the bottom centre of its bounding box goes
to `p`, like `base_at`), `car.rotate_by((0, deg(90), 0))` (turns it like one object around that point, wheels and tilted
parts included; `car.pivot(p)` picks another point), `car.scale_by(2)`. `car.create()` / `car.write()` build the parts up
one after another (3 s at most); `car.fade_in()` `car.fade_out()` `car.opacity_to(a)` `car.uncreate()` act on all at once.
Each returns an `Anim`, so `.duration()` `.ease()` `.delay()` and `all([..])` work. `s.place(&car, p)` moves it
instantly, and a group works with `create_all(&car)` and `clear(&car)`.

## Layout
`s.put(obj, x, y)` adds `obj` standing on screen pixel `(x, y)` (text and flat shapes are centred there):
`s.put(title("Hello"), 640, 80)`, `s.put(label("note"), 200, 600)`. Defaults: `label` is size 32 stone-gray and
steady, `title` is size 72 indigo and `.animate()`d; override with the usual builders. (A variable named
`title` or `label` hides the function of that name for the rest of its scope, so call the second one something else.)
`s.tag(id, "compute", Side::Above)` puts a label beside an object (`Above` `Below` `Left` `Right`), spaced to
fit the text; pass a styled `label("x").color("rose")` instead of a string to choose its look.
`s.heading("Title", "caption")` adds a header and caption near the top, sized to the canvas (the title only wobbles
on canvases big enough for it), and returns their ids
(`""` for no caption). A `title(..)` or `label(..)` added with plain `s.add(..)` and no `.at(..)` goes to the same
spots: top centre, and just under it.
`s.screen(point)` is the pixel a world point lands on; `obj.dimensions()` is a text object's size in pixels.

## Gotchas
- For a dark page use a theme. Setting only `.background("#101018")` leaves lines and text dark, so they vanish.
- To move several things together use a `Group`. Calling `move_to` on each part would stack them all on one point.
- `create`/`write`/`fade_in` hide their object until they start, so just `add` it (likewise the target of
  `transform_into`). Other animations don't.
- A `Stage` plays **once**: build a fresh one per video. A `.png` (last frame) can follow a video from the same
  stage, but a video after a `.png` is an error.
- Text is crisp and steady by default (solid fill, no wobble, written on with a wipe). `.animate()` gives it the
  lively hand-drawn wobble, which looks great on a big title and is hard to read when small. `.solid()` and
  `.hatched()` switch the fill; `.stroke_width(w)` adds a sketchy outline.
- `.handwritten()` makes text, `math(..)` or `latex(..)` look written by hand: wobbly, slightly uneven letters, a
  drifting baseline and a pen's weight. It is steady (no flicker) and stays readable, even small. Plain LaTeX is
  crisp typeset math. Combine freely with `.animate()`, `.tracking()` and colours.
- `.duration()` is the *total* length
- `.duration()` is the *total* length, so put `.delay()` after it. `.ease()` only affects single animations.
- Nearer objects hide those behind them automatically; stack with `base_at` using the lower object's top y.
- `.mp4`/`.webm`/`.gif` need `ffmpeg`; `math`/`latex` need `pdflatex` plus `pdftocairo` or `mutool`.
- Lower-level control (`Scene`, `Region`, custom `Geometry`, `Stage::with`) lives in the modules, not the prelude.
<!-- cheatsheet:end -->
