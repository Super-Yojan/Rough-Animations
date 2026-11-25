export const REPO = 'https://github.com/Super-Yojan/Rough-Animations';

export const features = [
	{ icon: '▱', color: 'var(--purple)', title: 'Isometric solids', body: 'Cuboids, cylinders, cones, spheres and extruded prisms. Nearer objects hide the ones behind them, so stacking just works.', code: 'cube(70).base_at((0,0,0))\nsphere(36).fill("sky")' },
	{ icon: '∪', color: 'var(--mint)', title: 'Boolean shapes', body: 'Combine flat shapes with union, intersect, minus and xor, then extrude the result into a solid with holes.', code: 'prism(rect(80,80)\n  .minus(circle(25)), 30)' },
	{ icon: '↝', color: 'var(--amber)', title: 'Morphing', body: 'Turn any flat shape, or any extruded solid, into any other. Outlines are paired and interpolated for you.', code: 'morph(a, star(5, 70, 30))\n  .ease(Easing::BackOut)' },
	{ icon: '∫', color: 'var(--rose)', title: 'Handwritten LaTeX', body: 'Formulas typeset by LaTeX and written on letter by letter. Add .handwritten() for a pen-drawn look.', code: 'math(r"\\int_0^1 x^2\\,dx")?\n  .handwritten()' },
	{ icon: '◎', color: '#82c3ff', title: 'Composable motion', body: 'Animations are values. Tweak them with .duration() and .ease(), group them with all, seq and lag.', code: 'lag(0.3, [move_by(a, d),\n          move_by(b, d)])' },
	{ icon: '◫', color: '#9be0d6', title: 'Groups', body: 'Build a rover or a character from parts, then move, turn, scale and fade it as one. Each part stays an ordinary object.', code: 'rover.move_by((170, 0, 0))\nrover.rotate_by((0, deg(90), 0))' },
	{ icon: '◐', color: '#c9c3ff', title: 'Themes', body: 'Six built in: light, dark, chalkboard, blueprint, sepia and terminal. One call changes the page and every default colour.', code: 'Stage::new(900, 600)\n  .theme("chalkboard")' },
	{ icon: '⌖', color: '#ffd2a8', title: 'Layout in pixels', body: 'Place titles and labels by screen pixel, tag objects and add a heading, with no world-coordinate arithmetic.', code: 's.put(title("Hello"), 640, 80)\ns.tag(id, "hub", Side::Above)' },
	{ icon: '✎', color: '#fad75a', title: 'Agent friendly', body: 'Scenes are a handful of lines, and the repo ships a skill that teaches the draw-a-still-first workflow. Install it with one command.', code: '/plugin install rough-animations \\\n  --marketplace Super-Yojan/rough-animations' }
];

export const steps = [
	{ title: 'Add', body: 'Every shape is one function returning an Object. Chain builders to place and colour it.', code: 'let a = s.add(circle(50).fill("purple"));' },
	{ title: 'Play', body: 'Animations are values you build, tweak, combine and hand to play.', code: 's.play(create(a));\ns.play(morph(a, star(5, 70, 30)));' },
	{ title: 'Render', body: 'The extension picks the output: mp4, webm or gif for video, png for the last frame.', code: 's.render("out.mp4")' }
];

export const heroCode = `use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600);
    let tower = s.add(
        cuboid(80, 110, 80).base_at((0, 0, 0)).fill("purple"),
    );
    let note = s.add(text("compute").at((0, 150, 0)));
    s.play(create(tower));   // outline traced, then filled
    s.play(write(note));
    s.play(all([
        move_by(tower, (100, 0, 0)),
        fill_to(tower, "rose"),
    ]).duration(1.5));
    s.wait(0.5);
    s.render("out.mp4")      // or .webm .gif .png
}`;

export const examples = [
	{
		value: 'shapes', label: 'Shapes', file: 'shapes.rs', img: 'shapes.png', cap: 'shapes.png', alt: 'Solids and flat shapes',
		code: `use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600);
    s.add(cube(70).base_at((-150, 0, 0)).fill("purple"));
    s.add(sphere(36).base_at((-40, 0, 0)).fill("sky"));
    // Combine flat shapes, then extrude the result into a solid with a hole.
    let plate = rect(80, 80).minus(circle(25));
    s.add(prism(plate, 30).base_at((90, 0, 0)).fill("rose"));
    s.render("shapes.png")
}`
	},
	{
		value: 'animation', label: 'Animation', file: 'anim.rs', img: 'occlusion.png', cap: 'occlusion.png · nearer objects hide those behind', alt: 'Objects hide each other',
		code: `use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600);
    let a = s.add(circle(50).fill("purple"));
    let b = s.add(cuboid(60, 60, 60).base_at((120, 0, 0)).fill("mint"));
    s.play(all([create(a), create(b)]).duration(1.2));
    s.play(morph(a, star(5, 70, 30)).ease(Easing::BackOut));       // circle into a star
    s.play(lag(0.3, [move_by(a, (0, 0, 80)), move_by(b, (0, 0, 80))]));
    s.play(rotate_by(b, (0, deg(90), 0)).duration(1).delay(0.5));  // duration first, then delay
    s.render("anim.mp4")
}`
	},
	{
		value: 'groups', label: 'Groups', file: 'rover.rs', img: 'rover.png', cap: 'rover.png · a Group that drives and turns as one', alt: 'A rover built as a group',
		code: `use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600);
    let mut rover = Group::new();
    s.add_to(&mut rover, cuboid(120, 30, 70).base_at((0, 10, 0)).fill("white"));
    s.add_to(&mut rover, cylinder(14, 10).at((40, 14, 40)).rotated((deg(90), 0, 0)).fill("ink"));
    s.add_to(&mut rover, cylinder(14, 10).at((-40, 14, 40)).rotated((deg(90), 0, 0)).fill("ink"));

    s.play(rover.create());                          // the parts build up one after another
    s.play(rover.move_by((80, 0, 0)).duration(2));   // the whole rover drives
    s.play(rover.rotate_by((0, deg(90), 0)));        // and turns on the spot, wheels included
    s.render("rover.mp4")
}`
	},
	{
		value: 'text', label: 'Text & LaTeX', file: 'hello.rs', img: 'text.png', cap: 'text.png', alt: 'Text and LaTeX',
		code: `use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(1280, 720).zoom(1.4);
    let title = s.add(text("Hello").font_size(90).color("#3c3278").animate().at(s.pixel(640, 120)));
    let formula = s.add(math(r"\\int_0^1 x^2\\,dx = \\frac{1}{3}")?.font_size(70).color("rose").handwritten().at(s.pixel(640, 320)));
    s.play(write(title));
    s.play(write(formula).duration(2));
    s.render("hello.mp4")
}`
	},
	{
		value: 'themes', label: 'Themes', file: 'themes.rs', img: 'themes.png', cap: 'themes.png · the same scene in each built-in theme', alt: 'The same scene in six themes',
		code: `use rough_animations::prelude::*;

fn main() -> Result {
    // One word changes the page and every default colour: "dark", "chalkboard",
    // "blueprint", "sepia", "terminal" or "light".
    let mut s = Stage::new(900, 600).theme("chalkboard").wobble(8);
    s.heading("Chalkboard", "the same script works in any theme");
    let c = s.add(cube(80).base_at((0, 0, 0)).fill("mint"));
    s.play(create(c));
    s.render("chalk.mp4")
}`
	}
];

export const webCode = `import { figure, cuboid, sphere, create, lag,
         FigurePlayer } from "rough-figure";

const spec = figure({
  width: 720, height: 520, zoom: 1.4,
  objects: [
    cuboid(150, 28, 150, { id: "base", baseAt: [0, 0, 0] }),
    sphere(24, { id: "orb", baseAt: [0, 28, 0],
                 fill: "sky", action: "ping" }),
  ],
  timeline: [lag(0.3, [create("base"), create("orb")])],
});

FigurePlayer.create(canvas, spec, {
  onAction: ({ action }) => console.log(action),
});`;

export const webFeatures = [
	{ icon: '↻', color: 'var(--purple)', title: 'Drag to turn', body: 'Real 3D rotation: depth order, hidden faces and wall decals follow the camera, with a fling when you let go.' },
	{ icon: '☝', color: 'var(--mint)', title: 'Clickable parts', body: 'Give a part an action and it highlights on hover and reports its name when clicked.' },
	{ icon: '▭', color: 'var(--amber)', title: 'A figure library', body: 'Eleven ready-made figures (empty inbox, 404, loader, success, ...) as typed React components or <rough-figure figure="...">, with a browsable catalog.' }
];

export const gallery = [
	['shapes.png', 'Solids, extruded shapes and arrows · examples/shapes.rs'],
	['booleans.png', 'union, intersect, minus, xor · examples/shapes.rs'],
	['occlusion.png', 'Stacking and hidden lines · examples/occlusion.rs'],
	['text.png', 'Labels, floor text and LaTeX · examples/text.rs'],
	['themes.png', 'The same scene in every theme · examples/themes.rs'],
	['rover.png', 'A rover built as a Group · examples/rover.rs']
];

export const agentCode = `# install the skill (Claude Code)
/plugin install rough-animations --marketplace Super-Yojan/rough-animations

# then look at what you made
S=~/.claude/skills/rough-animations/scripts   # wherever the skill lives
$S/contact-sheet.sh target/out.mp4 1.5 4 target/sheet.png
$S/frame.sh target/out.mp4 -0.5 target/end.png`;
