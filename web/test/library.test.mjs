// Every library figure is checked against the real engine (the same wasm the browser runs).
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { initSync, Figure } from "../dist/wasm/rough_engine.js";
import { library, categories } from "../dist/library/index.js";
import { toJSON } from "../dist/spec.js";

initSync({ module: readFileSync(new URL("../dist/wasm/rough_engine_bg.wasm", import.meta.url)) });

/** Everything of one key found anywhere in a nested spec. */
function collect(node, key, out = []) {
  if (Array.isArray(node)) node.forEach((n) => collect(n, key, out));
  else if (node && typeof node === "object") {
    for (const [k, v] of Object.entries(node)) {
      if (k === key) Array.isArray(v) ? out.push(...v) : out.push(v);
      collect(v, key, out);
    }
  }
  return out;
}

test("ids, names and categories are unique and valid", () => {
  const ids = library.map((f) => f.id);
  assert.equal(new Set(ids).size, ids.length, "duplicate id");
  assert.equal(new Set(library.map((f) => f.name)).size, library.length, "duplicate component name");
  const known = new Set(categories.map((c) => c.id));
  for (const f of library) {
    assert.match(f.id, /^[a-z0-9]+(-[a-z0-9]+)*$/, f.id);
    assert.match(f.name, /^[A-Z][A-Za-z0-9]*$/, f.name);
    assert.ok(known.has(f.category), `${f.id}: unknown category ${f.category}`);
    assert.ok(f.description.length > 20 && f.tags.length > 0, `${f.id}: needs a description and tags`);
  }
});

test("each figure declares exactly the actions its parts use", () => {
  for (const f of library) {
    const used = new Set(collect(f.spec.objects, "action"));
    const declared = new Set(f.actions.map((a) => a.name));
    assert.deepEqual([...used].sort(), [...declared].sort(), f.id);
  }
});

test("each timeline only names objects that exist", () => {
  for (const f of library) {
    const ids = new Set(collect(f.spec.objects, "id"));
    for (const t of [...collect(f.spec.timeline, "target"), ...collect(f.spec.timeline, "targets")]) {
      assert.ok(ids.has(t), `${f.id}: timeline names missing object "${t}"`);
    }
  }
});

test("each figure builds, plays to the end and draws something", () => {
  for (const f of library) {
    const fig = new Figure(toJSON(f.spec), 1);
    assert.ok(fig.duration() > 0 || f.id === "loading", `${f.id}: no timeline`);
    fig.seek(fig.duration());
    const px = fig.render();
    assert.equal(px.length, fig.width() * fig.height() * 4, f.id);
    const bg = px.slice(0, 4);
    let inked = 0;
    for (let i = 0; i < px.length; i += 4) if (px[i] !== bg[0] || px[i + 1] !== bg[1] || px[i + 2] !== bg[2]) inked++;
    assert.ok(inked > 2000, `${f.id}: looks empty (${inked} inked pixels)`);
    fig.free();
  }
});

test("every declared action can actually be clicked, at the start turn and turned around", () => {
  for (const f of library) {
    for (const yaw of [0, 120]) {
      const fig = new Figure(toJSON(f.spec), 1);
      fig.seek(fig.duration());
      if (yaw) fig.set_yaw(fig.yaw() + yaw);
      const found = new Set();
      for (let y = 0; y < fig.height(); y += 4) {
        for (let x = 0; x < fig.width(); x += 4) {
          const hit = fig.pick(x, y);
          if (hit) { const a = JSON.parse(hit).action; if (a) found.add(a); }
        }
      }
      // After turning, parts can be hidden behind others, so only the start turn must find all.
      const missing = f.actions.map((a) => a.name).filter((n) => !found.has(n));
      if (yaw === 0) assert.deepEqual(missing, [], `${f.id}: can't click ${missing}`);
      else assert.ok(found.size > 0 || f.actions.length === 0, `${f.id}: nothing clickable after turning`);
      fig.free();
    }
  }
});

test("every figure survives every theme and look", () => {
  for (const f of library) {
    const fig = new Figure(toJSON(f.spec), 2);
    fig.seek(fig.duration() / 2);
    for (const theme of ["light", "dark", "chalkboard", "blueprint", "sepia", "terminal"]) fig.set_theme(theme);
    fig.set_look("clean");
    assert.equal(fig.render().length, fig.width() * fig.height() * 4);
    fig.free();
  }
});
