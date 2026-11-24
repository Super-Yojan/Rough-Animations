use crate::world::bounds::Aabb;
use crate::planar::{Region, signed_area};
use crate::primitives::common::{faces_viewer, shading};
use crate::primitives::flat::Plane;
use crate::primitives::{DrawCtx, Geometry};
use crate::world::style::{Style, shade};
use crate::world::transform::{Transform, Vec3};
use tiny_skia::Color;

/// A [`Region`] extruded along local Y, centred on the origin. This is how boolean results
/// become solids: `Prism::new(&Region::rect(80., 80.) - &Region::circle(25.), 40.)` is a block
/// with a round hole through it.
///
/// The region lies in the XZ plane like [`Plane::Ground`] (`(u, v)` maps to `(x, -z)`).
#[derive(Clone, Debug)]
pub struct Prism {
    pub region: Region,
    pub height: f32,
}

impl Prism {
    pub fn new(region: Region, height: f32) -> Self {
        Self { region, height }
    }
}

struct Face {
    pts: Vec<Vec3>,
    normal: Vec3,
}

/// Turns sharper than this (degrees) get a vertical edge line; gentler ones are a smooth wall.
const CORNER_ANGLE: f32 = 25.0;

/// Visible runs of a contour's edges, as `(first_vertex, last_vertex)` index ranges
/// (inclusive, `last` may wrap past the end). `None` when every edge is visible.
fn visible_runs(vis: &[bool]) -> Option<Vec<(usize, usize)>> {
    let n = vis.len();
    if vis.iter().all(|&v| v) {
        return None;
    }
    let mut runs = Vec::new();
    let Some(start) = (0..n).find(|&k| vis[k] && !vis[(k + n - 1) % n]) else {
        return Some(runs);
    };
    let mut k = 0;
    while k < n {
        let e = (start + k) % n;
        if vis[e] {
            let first = e;
            let mut last = e;
            while k + 1 < n && vis[(start + k + 1) % n] {
                k += 1;
                last = (start + k) % n;
            }
            // Edge `last` ends at vertex `last + 1`.
            runs.push((first, last + 1));
        }
        k += 1;
    }
    Some(runs)
}

fn turn_angle(prev: [f32; 2], cur: [f32; 2], next: [f32; 2]) -> f32 {
    let (a, b) = ((cur[0] - prev[0], cur[1] - prev[1]), (next[0] - cur[0], next[1] - cur[1]));
    let cross = a.0 * b.1 - a.1 * b.0;
    let dot = a.0 * b.0 + a.1 * b.1;
    cross.atan2(dot).abs().to_degrees()
}

impl Geometry for Prism {
    fn bounds(&self, t: &Transform) -> Option<Aabb> {
        let (lo, hi) = self.region.bounds()?;
        let h = self.height / 2.0;
        // The region maps (u, v) -> (x, -z), like Plane::Ground.
        Some(Aabb::of_local_box(
            t,
            Vec3::new(lo[0], -h, -hi[1]),
            Vec3::new(hi[0], h, -lo[1]),
        ))
    }

    fn region(&self) -> Option<&Region> {
        Some(&self.region)
    }

    fn region_mut(&mut self) -> Option<&mut Region> {
        Some(&mut self.region)
    }

    fn draw(&self, ctx: &mut DrawCtx, t: &Transform, style: &Style) {
        let h = self.height / 2.0;
        let at = |p: [f32; 2], y: f32| {
            let v = Plane::Ground.to_3d(p);
            t.apply(Vec3::new(v.x, y, v.z))
        };

        // Side walls: one quad per contour edge, with the outward normal pointing away from
        // the solid (into the hole, for hole contours). Curved walls are made of many tiny
        // quads, so quads are filled without outlines; outlines are drawn separately below.
        let mut sides: Vec<Face> = Vec::new();
        // Per contour: its vertices and which edges face the viewer.
        let mut outlines: Vec<(&Vec<[f32; 2]>, Vec<bool>)> = Vec::new();
        for (i, c) in self.region.contours.iter().enumerate() {
            if c.len() < 3 {
                continue;
            }
            let flip = if self.region.contour_depth(i) % 2 == 0 { 1.0 } else { -1.0 };
            let orient = signed_area(c).signum() * flip;
            let mut vis = Vec::with_capacity(c.len());
            for k in 0..c.len() {
                let (a, b) = (c[k], c[(k + 1) % c.len()]);
                let (du, dv) = (b[0] - a[0], b[1] - a[1]);
                let local = Plane::Ground.to_3d([dv * orient, -du * orient]);
                let n = t.rotate_dir(local).normalize();
                let visible = faces_viewer(n);
                vis.push(visible);
                if visible {
                    sides.push(Face {
                        pts: vec![at(a, -h), at(b, -h), at(b, h), at(a, h)],
                        normal: n,
                    });
                }
            }
            outlines.push((c, vis));
        }
        let depth = |f: &Face| {
            let m = f.pts.iter().fold(Vec3::zero(), |s, p| s + *p) / 4.0;
            m.x + m.y + m.z
        };
        sides.sort_by(|a, b| depth(a).total_cmp(&depth(b)));

        let project_cap = |ctx: &DrawCtx, y: f32| -> Vec<Vec<_>> {
            self.region
                .contours
                .iter()
                .map(|c| c.iter().map(|&p| ctx.project(at(p, y))).collect())
                .collect()
        };
        let (bottom_cap, top_cap) = (project_cap(ctx, -h), project_cap(ctx, h));
        let bottom_n = t.rotate_dir(Vec3::new(0.0, -1.0, 0.0));
        let top_n = t.rotate_dir(Vec3::new(0.0, 1.0, 0.0));

        // Bottom cap (only if seen from below) first, then wall fills far to near, then the
        // wall outlines, top cap last.
        if faces_viewer(bottom_n) {
            let fill = style.fill.map(|c| shade(c, shading(bottom_n)));
            ctx.rough_path(&bottom_cap, fill, style);
        }
        let no_outline = Style {
            stroke: Color::from_rgba(0.0, 0.0, 0.0, 0.0).unwrap(),
            ..*style
        };
        for f in &sides {
            let pts: Vec<_> = f.pts.iter().map(|&p| ctx.project(p)).collect();
            let fill = style.fill.map(|c| shade(c, shading(f.normal)));
            ctx.rough_polygon(&pts, fill, &no_outline);
        }

        for (c, vis) in outlines {
            let n = c.len();
            let line = |ctx: &mut DrawCtx, y0: f32, y1: f32, v: usize| {
                let p = c[v % n];
                let pts = [ctx.project(at(p, y0)), ctx.project(at(p, y1))];
                ctx.rough_polyline(&pts, style);
            };
            let corner = |v: usize| {
                turn_angle(c[(v + n - 1) % n], c[v % n], c[(v + 1) % n]) > CORNER_ANGLE
            };
            match visible_runs(&vis) {
                // Every wall faces us: closed top and bottom loops.
                None => {
                    for y in [-h, h] {
                        let mut pts: Vec<_> = c.iter().map(|&p| ctx.project(at(p, y))).collect();
                        pts.push(pts[0]);
                        ctx.rough_polyline(&pts, style);
                    }
                    for v in (0..n).filter(|&v| corner(v)) {
                        line(ctx, -h, h, v);
                    }
                }
                Some(runs) => {
                    for (first, last) in runs {
                        for y in [-h, h] {
                            let pts: Vec<_> = (first..=last)
                                .map(|v| ctx.project(at(c[v % n], y)))
                                .collect();
                            ctx.rough_polyline(&pts, style);
                        }
                        // Silhouette ends always; interior vertices only at real corners.
                        for v in first..=last {
                            if v == first || v == last || corner(v) {
                                line(ctx, -h, h, v);
                            }
                        }
                    }
                }
            }
        }

        if faces_viewer(top_n) {
            let fill = style.fill.map(|c| shade(c, shading(top_n)));
            ctx.rough_path(&top_cap, fill, style);
        }
    }
}
