//! The WebAssembly entry point. The TypeScript package in `web/` wraps [`Figure`].
//!
//! A `Figure` holds one scene (built from a JSON [`Spec`]) and answers four questions from the
//! page: draw it at time `t`, draw it turned by some angle, what is under the pointer, and how
//! long is it. The page owns the timing, the canvas and the input; nothing here touches the DOM.
//!
//! Build with `scripts/build-web.sh`.

use crate::api::Stage;
use crate::spec::{Built, Spec};
use crate::world::arena::Id;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Figure {
    spec: Spec,
    stage: Stage,
    names: Vec<(Id, String)>,
    /// Turn in radians; kept here so a rebuild (rewind) restores it.
    yaw: f32,
    /// Device pixels per figure pixel.
    ratio: f32,
    /// The part drawn with a heavier outline because the pointer is over it.
    hover: Option<Id>,
}

fn err(e: String) -> JsError {
    JsError::new(&e)
}

#[wasm_bindgen]
impl Figure {
    /// Parse a figure description (see `spec.rs`) and build its scene. Throws a readable
    /// error for a bad description.
    ///
    /// `pixel_ratio` is the device pixels per figure pixel (`window.devicePixelRatio`): the
    /// figure is drawn that many times bigger with everything scaled to match, so it is sharp
    /// on high-density screens. `width()` and `height()` then report device pixels.
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str, pixel_ratio: Option<f32>) -> Result<Figure, JsError> {
        let spec = Spec::parse(json).map_err(err)?;
        let ratio = pixel_ratio.unwrap_or(1.0);
        let Built { stage, names } = spec.build_at(ratio).map_err(err)?;
        let yaw = stage.current_yaw();
        Ok(Figure { spec, stage, names, yaw, ratio, hover: None })
    }

    pub fn width(&self) -> u32 {
        self.stage.width
    }

    pub fn height(&self) -> u32 {
        self.stage.height
    }

    /// Length of the timeline in seconds.
    pub fn duration(&self) -> f32 {
        self.stage.duration()
    }

    /// The time the scene is currently at.
    pub fn time(&self) -> f32 {
        self.stage.time()
    }

    /// Turn of the scene in degrees.
    pub fn yaw(&self) -> f32 {
        self.yaw.to_degrees()
    }

    /// Turn the scene about the vertical axis. Takes effect on the next `render`.
    pub fn set_yaw(&mut self, degrees: f32) {
        self.yaw = degrees.to_radians();
        self.stage.set_yaw(self.yaw);
    }

    /// Move to time `t` (clamped to the timeline). Going backwards rebuilds the scene and plays
    /// it forward again, which costs the same as one forward seek of that length. It does not
    /// draw anything.
    pub fn seek(&mut self, t: f32) -> Result<(), JsError> {
        let t = t.clamp(0.0, self.stage.duration());
        if t < self.stage.time() {
            self.rebuild().map_err(err)?;
        }
        self.stage.seek(t);
        Ok(())
    }

    /// Draw the scene as it is now: straight RGBA, `width * height * 4` bytes, ready for
    /// `new ImageData(bytes, width, height)`.
    pub fn render(&mut self) -> Vec<u8> {
        // The hovered part gets a heavier outline for this one frame, then goes back, so a
        // running animation never fights the highlight.
        let lifted = self.hover.and_then(|id| {
            let o = self.stage.scene.get_mut(id)?;
            let before = o.style.stroke_width;
            o.style.stroke_width = before * 2.2 + 0.8;
            Some((id, before))
        });
        let mut data = self.stage.redraw().take();
        if let Some((id, before)) = lifted {
            if let Some(o) = self.stage.scene.get_mut(id) {
                o.style.stroke_width = before;
            }
        }
        // The engine works in premultiplied alpha; canvas wants straight alpha. The page
        // background is opaque unless the figure asked for a transparent one.
        for px in data.chunks_exact_mut(4) {
            let a = px[3] as u32;
            if a != 0 && a != 255 {
                for c in &mut px[..3] {
                    *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        data
    }

    /// The scene as it is now as an SVG document (vector paths, same strokes as `render`).
    /// Seek to the end first for a poster of the finished figure.
    pub fn to_svg(&mut self) -> String {
        self.stage.to_svg()
    }

    /// What is under screen pixel `(x, y)`: `null`, or JSON `{"id": name|null, "action": name|null}`
    /// for the topmost object there. Objects are named by the `id` they were given.
    pub fn pick(&self, x: f32, y: f32) -> Option<String> {
        let (hit, action) = self.stage.pick(x, y)?;
        let id = self.names.iter().find(|(i, _)| *i == hit).map(|(_, n)| n.as_str());
        Some(serde_json::json!({ "id": id, "action": action }).to_string())
    }

    /// Highlight the part with this `id` (or clear the highlight with `null`). Takes effect on
    /// the next `render`.
    pub fn set_hover(&mut self, id: Option<String>) {
        self.hover = id.and_then(|n| self.names.iter().find(|(_, x)| *x == n).map(|(i, _)| *i));
    }

    /// Switch theme (`light`, `dark`, `chalkboard`, `blueprint`, `sepia`, `terminal`), keeping
    /// the current time and turn.
    pub fn set_theme(&mut self, name: &str) -> Result<(), JsError> {
        self.spec.theme = Some(name.to_string());
        self.rebuild_in_place().map_err(err)
    }

    /// `"sketch"` or `"clean"`.
    pub fn set_look(&mut self, look: &str) -> Result<(), JsError> {
        self.spec.look = Some(look.to_string());
        self.rebuild_in_place().map_err(err)
    }

    /// Replace the page colour (`"#101018"`, a name, or `"transparent"`).
    pub fn set_background(&mut self, color: &str) -> Result<(), JsError> {
        self.spec.background = Some(color.to_string());
        self.rebuild_in_place().map_err(err)
    }
}

impl Figure {
    fn rebuild(&mut self) -> Result<(), String> {
        let Built { mut stage, names } = self.spec.build_at(self.ratio)?;
        stage.set_yaw(self.yaw);
        self.stage = stage;
        self.hover = self.hover.and_then(|old| {
            // Ids are fresh after a rebuild: find the same name again.
            let name = self.names.iter().find(|(i, _)| *i == old)?.1.clone();
            names.iter().find(|(_, n)| *n == name).map(|(i, _)| *i)
        });
        self.names = names;
        Ok(())
    }

    /// Rebuild and return to the time we were at.
    fn rebuild_in_place(&mut self) -> Result<(), String> {
        let t = self.stage.time();
        self.rebuild()?;
        self.stage.seek(t);
        Ok(())
    }
}
