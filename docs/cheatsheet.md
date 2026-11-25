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

**Turning, clicking, vectors:** `.yaw(deg(30))` turns the whole scene about the vertical axis (`s.set_yaw(radians)` to
change it later); solids, hidden-surface order and wall decals follow. `obj.action("open")` names a part, and
`s.pick(x, y)` returns `Some((id, action))` for the topmost part under a screen pixel. `.pixel_ratio(2.0)` draws for
high-density screens (pass the device size to `Stage::new`, and scale `zoom`/`origin` by the same ratio).
`s.to_svg()` returns the current frame as an SVG string (finished frames; a half-drawn shape is written whole).

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
