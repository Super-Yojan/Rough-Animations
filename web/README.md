# rough-figure

Interactive hand-drawn 3D figures for the web. The drawing is done live by the
[rough-animations](../README.md) Rust engine compiled to WebAssembly; this package is the TypeScript
layer on top: typed scene builders, a canvas player, a `<rough-figure>` element and a React component.

- **Drag to turn** the scene about the vertical axis (with a fling), arrow keys too.
- **Hover and click parts.** Give an object an `action` and it highlights on hover and reports
  its name when clicked.
- **Timeline you can scrub, pause, loop and replay.**
- Themes (`light`, `dark`, `chalkboard`, `blueprint`, `sepia`, `terminal`), and `sketch` or `clean` lines.
- Draws only when something changes, and stops while scrolled out of view.

## Build

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version "$(grep -A1 'name = "wasm-bindgen"$' ../Cargo.lock | grep version | cut -d'"' -f2)" --locked
pnpm install
pnpm build:wasm      # Rust -> web/src/wasm
pnpm build           # TypeScript -> dist (and copies the wasm next to it)
pnpm demo            # http://localhost:5173/catalog/  (also /demo/ and /demo/gallery.html)
```

## Use it

### Write a figure in TypeScript

```ts
import { figure, cuboid, sphere, create, lag, FigurePlayer } from "rough-figure";

const spec = figure({
  width: 720, height: 520, zoom: 1.4, origin: [360, 360],
  objects: [
    cuboid(150, 28, 150, { id: "base", baseAt: [0, 0, 0], fill: "slate" }),
    sphere(24, { id: "orb", baseAt: [0, 28, 0], fill: "sky", action: "ping" }),
  ],
  timeline: [lag(0.3, [create("base"), create("orb")])],
});

const player = await FigurePlayer.create(document.querySelector("canvas")!, spec, {
  loop: true,
  onAction: ({ action, id }) => console.log(action, id),
});
```

The builders mirror the Rust API (`cuboid`, `cylinder`, `prism`, `create`, `moveBy`, `lag`, ...).
Angles are in degrees. A mistake in a figure (unknown colour, missing field, unknown object name) throws
an error with a readable message instead of drawing nothing.

### As an HTML element

```html
<script type="module" src="rough-figure/element"></script>

<rough-figure src="/figures/inbox.json" theme="dark" loop></rough-figure>
```

Attributes: `src` (or an inline `<script type="application/json">`), `theme`, `look`, `yaw`, `speed`,
`loop`, `static` (show the finished figure), `fixed` (no dragging or clicking). Events: `ready`,
`error`, `action` (`event.detail` is `{ action, id }`).

### In React

```tsx
import { RoughFigure } from "rough-figure/react";

<RoughFigure spec={spec} theme="dark" loop onAction={(e) => navigate(e.action)} />;
```

### Component library

Eleven ready-made figures for product UI, each a named component with typed props:

| Component (id) | Category | Actions |
|---|---|---|
| `EmptyInbox` (`empty-inbox`) | Empty states | `compose`, `open` |
| `Upload` (`upload`) | Empty states | `upload` |
| `NotFound` (`not-found`) | Errors | `home`, `search` |
| `AccessDenied` (`access-denied`) | Errors | `unlock` |
| `Loading` (`loading`) | Loaders | none |
| `Success` (`success`) | Status | `open` |
| `Notifications` (`notifications`) | Status | `open`, `dismiss` |
| `Servers` (`servers`) | Status | `unit-1`, `unit-2`, `unit-3` |
| `Analytics` (`analytics`) | Product | `report`, `trend` |
| `Settings` (`settings`) | Product | `settings`, `tune` |
| `Rocket` (`rocket`) | Onboarding | `launch`, `fins` |

```tsx
import { EmptyInbox } from "rough-figure/library/react";

<EmptyInbox loop size={420} onAction={(e) => e.action === "compose" && openComposer()} />;
//                                         ^ typed: "compose" | "open"
```

```html
<rough-figure figure="not-found" loop></rough-figure>
```

```ts
import { library, figureById } from "rough-figure/library";   // metadata: title, tags, actions, spec
```

Run `pnpm demo` and open `/catalog/` for a browsable catalog: search, category filter, theme switcher,
live previews, usage snippets (React, HTML element, TypeScript) and SVG/PNG download for each figure.

Add your own with `defineFigure({ id, name, title, description, category, tags, actions, spec })`.
`pnpm test` loads every figure into the real engine and checks that ids are unique, that the declared
actions are exactly the ones the parts use, that timelines only name objects that exist, that each
figure draws and survives every theme, and that every declared action can really be clicked. The
figures are starter quality: simple solids, easy to restyle, not finished illustrations.

(`rough-figure/figures` still exports the first four as plain specs.)

### Static posters (SVG and PNG)

```ts
player.seek(player.duration);        // the finished figure
const svg = player.toSVG();          // vector paths, same strokes and hatching, sharp at any size
const png = await player.toBlob();   // what is on the canvas right now
```

Use the SVG as the `<img>` fallback while the engine loads, for print, or for social images. A shape
caught mid-`create` is written whole, so export finished frames. With a transparent page, hidden-line
removal uses the theme's page colour.

### Player API

`play() pause() replay() seek(t) scrub(0..1) setYaw(deg) setTheme(name) setLook(look) toSVG() toBlob()
destroy()`, and `duration`, `time`, `playing`, `yaw`, `lastFrameMs`.

## Notes and limits

- Figures are drawn at `devicePixelRatio` (at most 3) so they stay sharp on retina screens; line
  widths, hatching and text scale with it. Cost grows with the square of the ratio, so pass
  `pixelRatio: 1` to the player for very large figures. `player.lastFrameMs` reports the draw time.
- Picking uses the screen outline of each object's bounding box, so round parts are hit a little
  beyond their drawn edge. Plain text is never picked.
- Rewinding rebuilds the scene and plays it forward again (cheap, nothing is drawn), so scrubbing
  backwards costs one forward seek of that length.
- LaTeX (`math`, `latex`) is not available in the browser build.
- The wasm file is about 1.7 MB before compression.
