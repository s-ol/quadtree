use glam::{Vec2, vec2};

#[cfg(feature = "serde")]
use serde::Serialize;

/// A trait for shapes that can be used to query the Quadtree. Shapes must be able to
/// provide their start and end points, their center point, and check if they contain
/// a point. They must also be able to check if they intersect with another shape.
pub trait Shape {
    /// Get the bounding rect of the shape
    fn aabb(&self) -> Rect;
    /// Check if the shape contains a point
    fn contains(&self, point: Vec2) -> bool;
    /// Check if the shape shares any space with another shape
    fn intersects(&self, other: &Self) -> bool;
    /// Check if the shape fully contains a given rect
    fn contains_rect(&self, rect: &Rect) -> bool {
        self.contains(rect.aa()) && self.contains(rect.bb())
    }
}

impl Shape for Vec2 {
    fn aabb(&self) -> Rect {
        Rect::new(*self, *self)
    }

    fn contains(&self, point: Vec2) -> bool {
        *self == point
    }

    fn intersects(&self, other: &Self) -> bool {
        self == other
    }
}

/// Represents an axis-aligned rectangle defined by two points: the start and the end.
/// It is used to define boundaries for Quadtree nodes and provides utility functions
/// for geometric calculations.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Rect {
    /// Start
    pub aa: Vec2,
    /// End
    pub bb: Vec2,
}

impl Rect {
    /// Create a new rect with a start and end point
    pub const fn new(a: Vec2, b: Vec2) -> Self {
        Self { aa: a, bb: b }
    }

    /// Get the point (aa.x, aa.y)
    pub const fn aa(&self) -> Vec2 {
        self.aa
    }

    /// Get the point (bb.x, bb.y)
    pub const fn bb(&self) -> Vec2 {
        self.bb
    }

    /// Get the point (aa.x, bb.y)
    pub const fn ab(&self) -> Vec2 {
        vec2(self.aa.x, self.bb.y)
    }

    /// Get the point (bb.x, aa.y)
    pub const fn ba(&self) -> Vec2 {
        vec2(self.bb.x, self.aa.y)
    }

    /// Get the midpoint of aa and bb
    pub fn center(&self) -> Vec2 {
        Vec2::midpoint(self.aa, self.bb)
    }

    /// Get the perimeter of the rect
    pub fn perimeter(&self) -> f32 {
        let diff = self.bb - self.aa;
        diff.x * 2.0 + diff.y * 2.0
    }

    /// Quarter the rect to produce four smaller rects
    pub fn quarter(&self) -> [Self; 4] {
        let center = self.center();
        let diff = center - self.aa;
        let diff_x = vec2(diff.x, 0.);
        let diff_y = vec2(0., diff.y);

        [
            Rect::new(self.aa, center),
            Rect::new(self.aa + diff_x, center + diff_x),
            Rect::new(self.aa + diff_y, center + diff_y),
            Rect::new(center, self.bb),
        ]
    }

    /// Determine quadrant index (z-order), or `None` if the point is outside of the rect
    pub fn quadrant(&self, point: Vec2) -> Option<usize> {
        if !self.contains(point) {
            return None;
        }

        let center = self.center();
        Some(((point.y > center.y) as usize) << 1 | (point.x > center.x) as usize)
    }
}

impl Shape for Rect {
    fn aabb(&self) -> Rect {
        *self
    }

    fn contains(&self, point: Vec2) -> bool {
        point.x >= self.aa.x && point.y >= self.aa.y && point.x <= self.bb.x && point.y <= self.bb.y
    }

    fn intersects(&self, other: &Self) -> bool {
        !(self.bb.x < other.aa.x
            || self.aa.x > other.bb.x
            || self.bb.y < other.aa.y
            || self.aa.y > other.bb.y)
    }
}

/// Represents a circle defined by a center point and radius. Provides utility functions
/// for geometric calculations, particularly for interactions with Quadtree.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    /// Create a new circle with a center point and radius
    pub const fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Shape for Circle {
    fn aabb(&self) -> Rect {
        let v = vec2(self.radius, self.radius);
        Rect::new(self.center - v, self.center + v)
    }

    fn contains(&self, point: Vec2) -> bool {
        Vec2::distance(self.center, point) <= self.radius
    }

    fn intersects(&self, other: &Self) -> bool {
        Vec2::distance(self.center, other.center) <= self.radius + other.radius
    }
}

#[cfg(test)]
mod tests {
    use crate::util::tests::{make_circle, make_rect};

    use super::*;

    #[test]
    fn rect_properties() {
        let rect = make_rect(0.0, 0.0, 10.0, 10.0);
        assert_eq!(rect.aa(), vec2(0.0, 0.0), "Start should be at (0.0, 0.0)");
        assert_eq!(rect.bb(), vec2(10.0, 10.0), "End should be at (10.0, 10.0)");
        assert_eq!(
            rect.center(),
            vec2(5.0, 5.0),
            "Center should be at (5.0, 5.0)"
        );
    }

    #[test]
    fn rect_contains_point() {
        let rect = make_rect(0.0, 0.0, 10.0, 10.0);
        assert!(
            rect.contains(vec2(5.0, 5.0)),
            "Rect should contain point (5.0, 5.0)"
        );
        assert!(
            !rect.contains(vec2(-1.0, 5.0)),
            "Rect should not contain point (-1.0, 5.0)"
        );
        assert!(
            rect.contains(vec2(0.0, 0.0)),
            "Rect should contain its start point (0.0, 0.0)"
        );
        assert!(
            rect.contains(vec2(10.0, 10.0)),
            "Rect should contain its end point (10.0, 10.0)"
        );
    }

    #[test]
    fn rect_intersects_with_another_rect() {
        let rect1 = make_rect(0.0, 0.0, 10.0, 10.0);
        let rect2 = make_rect(5.0, 5.0, 15.0, 15.0);
        assert!(
            rect1.intersects(&rect2),
            "Rect1 should intersect with Rect2"
        );

        let rect3 = make_rect(10.0, 10.0, 20.0, 20.0);
        assert!(
            rect1.intersects(&rect3),
            "Rect1 should touch Rect3 at the edge, counting as intersect"
        );

        let rect4 = make_rect(11.0, 11.0, 21.0, 21.0);
        assert!(
            !rect1.intersects(&rect4),
            "Rect1 should not intersect with Rect4"
        );

        let rect5 = make_rect(-5.0, -5.0, -1.0, -1.0);
        assert!(
            !rect1.intersects(&rect5),
            "Rect1 should not intersect with Rect5"
        );

        let rect6 = make_rect(3.0, 3.0, 7.0, 7.0);
        assert!(
            rect1.intersects(&rect6),
            "Rect6 is entirely inside Rect1, should intersect"
        );

        let rect7 = make_rect(-10.0, 0.0, -1.0, 10.0);
        assert!(
            !rect1.intersects(&rect7),
            "Rect1 should not intersect with Rect7 on the left"
        );

        let rect8 = make_rect(0.0, -10.0, 10.0, -1.0);
        assert!(
            !rect1.intersects(&rect8),
            "Rect1 should not intersect with Rect8 below it"
        );

        let rect9 = make_rect(0.0, 10.0, 10.0, 20.0);
        assert!(
            rect1.intersects(&rect9),
            "Rect1 should touch Rect9 at the top, counting as intersect"
        );

        let rect10 = make_rect(10.0, 0.0, 20.0, 10.0);
        assert!(
            rect1.intersects(&rect10),
            "Rect1 should touch Rect10 on the right, counting as intersect"
        );

        let rect11 = make_rect(5.0, -5.0, 15.0, 5.0);
        assert!(
            rect1.intersects(&rect11),
            "Rect11 should intersect with the bottom part of Rect1"
        );
    }

    #[test]
    fn rect_contains_another_rect() {
        let outer_rect = make_rect(0.0, 0.0, 10.0, 10.0);
        let inner_rect = make_rect(1.0, 1.0, 9.0, 9.0);
        assert!(
            outer_rect.contains_rect(&inner_rect),
            "Outer rect should contain inner rect completely"
        );

        let outer_rect = make_rect(0.0, 0.0, 10.0, 10.0);
        let overlapping_rect = make_rect(5.0, 5.0, 15.0, 15.0);
        assert!(
            !outer_rect.contains_rect(&overlapping_rect),
            "Outer rect should not contain overlapping rect"
        );
    }

    #[test]
    fn rect_perimeter() {
        let rect = make_rect(0.0, 0.0, 10.0, 5.0);
        assert_eq!(rect.perimeter(), 30.0, "Perimeter should be 30.0");
    }

    #[test]
    fn quartering_rect() {
        let rect = make_rect(0.0, 0.0, 10.0, 10.0);
        let quarters = rect.quarter();
        // Ensure each quarter is correctly sized and positioned
        assert_eq!(
            quarters[0],
            make_rect(0.0, 0.0, 5.0, 5.0),
            "Top-left quarter should match expected dimensions"
        );
        assert_eq!(
            quarters[1],
            make_rect(5.0, 0.0, 10.0, 5.0),
            "Top-right quarter should match expected dimensions"
        );
        assert_eq!(
            quarters[2],
            make_rect(0.0, 5.0, 5.0, 10.0),
            "Bottom-left quarter should match expected dimensions"
        );
        assert_eq!(
            quarters[3],
            make_rect(5.0, 5.0, 10.0, 10.0),
            "Bottom-right quarter should match expected dimensions"
        );
    }

    #[test]
    fn test_determine_quadrant() {
        let rect = make_rect(0.0, 0.0, 10.0, 10.0);
        let points = [
            vec2(2.5, 2.5),   // Should be in the first quadrant (index 0)
            vec2(7.5, 2.5),   // Should be in the second quadrant (index 1)
            vec2(2.5, 7.5),   // Should be in the third quadrant (index 2)
            vec2(7.5, 7.5),   // Should be in the fourth quadrant (index 3)
            vec2(10.5, 10.5), // Should be outside all quadrants (None)
        ];

        let expected_quadrants = [Some(0), Some(1), Some(2), Some(3), None];
        let results = points
            .iter()
            .map(|point| rect.quadrant(*point))
            .collect::<Vec<_>>();

        assert_eq!(
            results, expected_quadrants,
            "Each point should match its expected quadrant"
        );
    }

    #[test]
    fn circle_properties_and_bounds() {
        let circle = make_circle(5.0, 5.0, 5.0);
        assert_eq!(
            circle.center,
            vec2(5.0, 5.0),
            "Center should be at (5.0, 5.0)"
        );
        assert_eq!(circle.radius, 5.0, "Radius should be 5.0");
        assert_eq!(
            circle.aabb(),
            make_rect(0.0, 0.0, 10.0, 10.0),
            "Rect should be (0.0, 0.0, 10.0, 10.0)"
        );
    }

    #[test]
    fn circle_contains_point() {
        let circle = make_circle(5.0, 5.0, 5.0);
        assert!(
            circle.contains(vec2(5.0, 5.0)),
            "Circle should contain its center point"
        );
        assert!(
            circle.contains(vec2(0.0, 5.0)),
            "Circle should contain point on its perimeter"
        );
        assert!(
            !circle.contains(vec2(0.0, 0.0)),
            "Circle should not contain points outside its boundary"
        );
    }

    #[test]
    fn circle_intersects_another_circle() {
        let circle1 = make_circle(5.0, 5.0, 5.0);
        let circle2 = make_circle(10.0, 5.0, 5.0);
        let circle3 = make_circle(20.0, 5.0, 5.0);

        assert!(
            circle1.intersects(&circle2),
            "Circle1 should intersect Circle2"
        );
        assert!(
            !circle1.intersects(&circle3),
            "Circle1 should not intersect Circle3"
        );
    }

    #[test]
    fn circle_contains_rect() {
        let circle = make_circle(5.0, 5.0, 5.0);
        let rect = make_rect(4.0, 4.0, 6.0, 6.0);
        assert!(circle.contains_rect(&rect), "Circle should contain rect");

        let circle = make_circle(5.0, 5.0, 5.0);
        let rect = make_rect(5.0, 5.0, 10.0, 10.0);
        assert!(
            !circle.contains_rect(&rect),
            "Circle should not contain overlapping rect"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_rect() {
        let rect = make_rect(0.0, 0.0, 10.0, 10.0);
        let json = serde_json::to_string(&rect).expect("Rect should serialize successfully");
        let expected_json = r#"{"start":[0.0,0.0],"end":[10.0,10.0]}"#;
        assert_eq!(json, expected_json, "Rect should serialize correctly");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_circle() {
        let circle = make_circle(5.0, 5.0, 5.0);
        let json = serde_json::to_string(&circle).expect("Circle should serialize successfully");
        let expected_json = r#"{"center":[5.0,5.0],"radius":5.0}"#;
        assert_eq!(json, expected_json, "Circle should serialize correctly");
    }
}
