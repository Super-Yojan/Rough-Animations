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
| `cuboid(w,h,d)` `cube(s)` | `circle(r)` `ellipse(rx,ry)` `rect(w,h)` | `line(a,b)` `arrow(a,b)` |
| `cylinder(r,h)` `cone(r,h)` `sphere(r)` | `ngon(sides,r)` `star(n,outer,inner)` `polygon([(x,y),..])` | `text("..")` `math(r"\frac12")?` `latex("..")?` |
| `prism(flat_shape, h)` extrudes a flat shape | booleans: `.union(b)` `.minus(b)` `.intersect(b)` `.xor(b)` | |

Builders: `.at(p)` `.base_at(p)` `.fill(c)` `.stroke(c)` `.color(c)` (fill+stroke) `.opacity(0..1)`
`.stroke_width(w)` `.rotated(r)` `.scaled(s)` `.layer(n)` `.hidden()`, flat shapes: `.shift(dx,dy)` `.on(plane)`,
text: `.font_size(n)`. Colours: `"purple"` `"mint"` `"amber"` `"sky"` `"rose"` `"gold"` `"slate"` `"ink"`
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

## Stage
`Stage::new(w,h)` then optional `.fps(n)` `.background(c)` `.origin(x,y)` `.wobble(fps)` (boiling lines)
`.sketch(Sketch::clean())`. `s.add(obj) -> Id`, `s.play(anim)`, `s.wait(secs)`, `s.render(path)`.

## Gotchas
- `create`/`write`/`fade_in` hide their object until they start, so just `add` it (likewise the target of
  `transform_into`). Other animations don't.
- A `Stage` plays **once**: build a fresh one per video. A `.png` (last frame) can follow a video from the same
  stage, but a video after a `.png` is an error.
- `.duration()` is the *total* length, so put `.delay()` after it. `.ease()` only affects single animations.
- Nearer objects hide those behind them automatically; stack with `base_at` using the lower object's top y.
- `.mp4`/`.webm`/`.gif` need `ffmpeg`; `math`/`latex` need `pdflatex` plus `pdftocairo` or `mutool`.
- Lower-level control (`Scene`, `Region`, custom `Geometry`, `Stage::with`) lives in the modules, not the prelude.
