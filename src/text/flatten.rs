use crate::planar::{Point2, Region};

/// Turns move/line/quad/cubic path commands into closed polyline contours.
pub struct Flattener {
    out: Vec<Vec<Point2>>,
    cur: Vec<Point2>,
    /// Target segment length when flattening curves, in the caller's units.
    seg_len: f32,
}

impl Flattener {
    pub fn new(seg_len: f32) -> Self {
        Self {
            out: Vec::new(),
            cur: Vec::new(),
            seg_len,
        }
    }

    pub fn move_to(&mut self, p: Point2) {
        self.finish();
        self.cur.push(p);
    }

    pub fn line_to(&mut self, p: Point2) {
        self.cur.push(p);
    }

    pub fn quad_to(&mut self, c: Point2, p: Point2) {
        let Some(&a) = self.cur.last() else {
            return self.move_to(p);
        };
        let n = self.steps(dist(a, c) + dist(c, p));
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            self.cur.push([
                u * u * a[0] + 2.0 * u * t * c[0] + t * t * p[0],
                u * u * a[1] + 2.0 * u * t * c[1] + t * t * p[1],
            ]);
        }
    }

    pub fn cubic_to(&mut self, c1: Point2, c2: Point2, p: Point2) {
        let Some(&a) = self.cur.last() else {
            return self.move_to(p);
        };
        let n = self.steps(dist(a, c1) + dist(c1, c2) + dist(c2, p));
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.cur.push([
                w0 * a[0] + w1 * c1[0] + w2 * c2[0] + w3 * p[0],
                w0 * a[1] + w1 * c1[1] + w2 * c2[1] + w3 * p[1],
            ]);
        }
    }

    pub fn close(&mut self) {
        self.finish();
    }

    pub fn into_region(mut self) -> Region {
        self.finish();
        Region::from_contours(self.out)
    }

    fn steps(&self, control_len: f32) -> usize {
        ((control_len / self.seg_len).ceil() as usize).clamp(2, 24)
    }

    fn finish(&mut self) {
        let mut c = std::mem::take(&mut self.cur);
        if c.len() > 1 && dist(c[0], c[c.len() - 1]) < 1e-6 {
            c.pop();
        }
        if c.len() >= 3 {
            self.out.push(c);
        }
    }
}

fn dist(a: Point2, b: Point2) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closes_and_drops_the_repeated_end_point() {
        let mut f = Flattener::new(1.0);
        f.move_to([0., 0.]);
        f.line_to([4., 0.]);
        f.line_to([4., 4.]);
        f.line_to([0., 0.]); // explicit return to start
        f.close();
        let r = f.into_region();
        assert_eq!(r.contours.len(), 1);
        assert_eq!(r.contours[0].len(), 3);
    }

    #[test]
    fn curves_are_subdivided_and_land_on_the_endpoint() {
        let mut f = Flattener::new(1.0);
        f.move_to([0., 0.]);
        f.quad_to([5., 10.], [10., 0.]);
        f.close();
        let r = f.into_region();
        let c = &r.contours[0];
        assert!(c.len() > 4);
        assert_eq!(*c.last().unwrap(), [10., 0.]);
    }

    #[test]
    fn degenerate_contours_are_dropped() {
        let mut f = Flattener::new(1.0);
        f.move_to([0., 0.]);
        f.line_to([1., 1.]);
        f.close();
        assert!(f.into_region().contours.is_empty());
    }
}
