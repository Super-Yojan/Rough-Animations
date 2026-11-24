use super::Region;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use std::ops::{BitAnd, BitOr, BitXor, Sub};

impl Region {
    fn overlay(&self, other: &Region, rule: OverlayRule) -> Region {
        let shapes = self
            .contours
            .overlay(&other.contours, rule, FillRule::EvenOdd);
        Region::from_contours(shapes.into_iter().flatten().collect())
    }

    /// Everything covered by `self` or `other`.
    pub fn union(&self, other: &Region) -> Region {
        self.overlay(other, OverlayRule::Union)
    }

    /// Only what both cover.
    pub fn intersection(&self, other: &Region) -> Region {
        self.overlay(other, OverlayRule::Intersect)
    }

    /// What `self` covers that `other` does not.
    pub fn difference(&self, other: &Region) -> Region {
        self.overlay(other, OverlayRule::Difference)
    }

    /// What exactly one of them covers (union minus intersection).
    pub fn xor(&self, other: &Region) -> Region {
        self.overlay(other, OverlayRule::Xor)
    }
}

/// `&a | &b` is the union.
impl BitOr for &Region {
    type Output = Region;
    fn bitor(self, rhs: &Region) -> Region {
        self.union(rhs)
    }
}

/// `&a & &b` is the intersection.
impl BitAnd for &Region {
    type Output = Region;
    fn bitand(self, rhs: &Region) -> Region {
        self.intersection(rhs)
    }
}

/// `&a - &b` is the difference.
impl Sub for &Region {
    type Output = Region;
    fn sub(self, rhs: &Region) -> Region {
        self.difference(rhs)
    }
}

/// `&a ^ &b` is the exclusive-or.
impl BitXor for &Region {
    type Output = Region;
    fn bitxor(self, rhs: &Region) -> Region {
        self.xor(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(x: f32, y: f32) -> Region {
        Region::rect(4.0, 4.0).translated(x, y) // covers x±2, y±2; area 16
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    // Two 4x4 squares overlapping in a 2x4 strip (area 8).
    #[test]
    fn union_intersection_difference_xor_areas() {
        let (a, b) = (sq(0.0, 0.0), sq(2.0, 0.0));
        assert!(close(a.union(&b).area(), 24.0));
        assert!(close(a.intersection(&b).area(), 8.0));
        assert!(close(a.difference(&b).area(), 8.0));
        assert!(close(b.difference(&a).area(), 8.0));
        assert!(close(a.xor(&b).area(), 16.0));
    }

    #[test]
    fn disjoint_shapes() {
        let (a, b) = (sq(0.0, 0.0), sq(10.0, 0.0));
        assert!(a.intersection(&b).is_empty());
        assert_eq!(a.union(&b).contours.len(), 2);
        assert!(close(a.difference(&b).area(), 16.0));
    }

    #[test]
    fn difference_can_punch_a_hole() {
        let donut = &Region::circle(10.0) - &Region::circle(4.0);
        assert_eq!(donut.contours.len(), 2);
        assert!(!donut.contains([0.0, 0.0]));
        assert!(donut.contains([7.0, 0.0]));
        let expect = Region::circle(10.0).area() - Region::circle(4.0).area();
        assert!(close(donut.area(), expect), "{} vs {expect}", donut.area());
    }

    #[test]
    fn operators_match_methods() {
        let (a, b) = (sq(0.0, 0.0), sq(2.0, 0.0));
        assert_eq!(&a | &b, a.union(&b));
        assert_eq!(&a & &b, a.intersection(&b));
        assert_eq!(&a - &b, a.difference(&b));
        assert_eq!(&a ^ &b, a.xor(&b));
    }

    #[test]
    fn results_can_be_combined_again() {
        // (circle ∪ circle) minus a rect through the middle.
        let two = &Region::circle(5.0) | &Region::circle(5.0).translated(6.0, 0.0);
        let cut = &two - &Region::rect(2.0, 100.0).translated(3.0, 0.0);
        assert!(cut.area() < two.area());
        assert!(!cut.contains([3.0, 0.0]));
        assert!(cut.contains([-3.0, 0.0]));
    }
}
