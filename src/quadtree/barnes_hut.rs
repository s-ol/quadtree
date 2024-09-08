use nalgebra as na;

use crate::{quadtree::Node, shapes::Shape, Mass, Point, QuadTree, P2};

/// A point with mass
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct WeightedPoint {
    pos: P2,
    mass: f64,
}

impl WeightedPoint {
    pub fn new(pos: P2, mass: f64) -> Self {
        Self { pos, mass }
    }
}

impl Point for WeightedPoint {
    fn point(&self) -> P2 {
        self.pos
    }
}

impl Mass for WeightedPoint {
    fn mass(&self) -> f64 {
        self.mass
    }
}

fn compute_cm<T: Point + Mass>(data: &[T]) -> WeightedPoint {
    let mut total_mass = 0.0;
    let mut weighted_sum = P2::origin();
    for x in data {
        let p = x.point();
        let m = x.mass();
        total_mass += m;
        weighted_sum += m * p.coords;
    }

    let point = weighted_sum / total_mass;
    WeightedPoint {
        pos: point,
        mass: total_mass,
    }
}

impl<T: Point + Mass> QuadTree<T> {
    /// Perform Barnes-Hut approximation for a single point
    ///
    /// An interactive explanation of the algorithm can be found [here](https://jheer.github.io/barnes-hut/)
    ///
    /// ### Arguments
    /// * `subject` - The point to approximate forces for
    /// * `theta` - The approximation threshold
    ///
    /// **Returns** a vector of approximated points to be used for force computation
    pub fn barnes_hut(&mut self, subject: &impl Point, theta: f64) -> Vec<WeightedPoint> {
        let mut results = Vec::with_capacity(self.count);
        self.root.approximate_points(subject, theta, &mut results);
        results
    }
}

impl<T: Point + Mass> Node<T> {
    /// Compute the center of mass of the node (point with no mass at the origin if no data)
    fn center_of_mass(&mut self) -> WeightedPoint {
        match self {
            Self::Empty { .. } => WeightedPoint::default(),
            Self::External { data, cm, .. } => match cm {
                Some(wp) => *wp,
                None => {
                    let wp = compute_cm(data);
                    *cm = Some(wp);
                    wp
                }
            },
            Self::Internal { children, cm, .. } => match cm {
                Some(wp) => *wp,
                None => {
                    let weighted_points = children
                        .iter_mut()
                        .map(|c| c.center_of_mass())
                        .collect::<Vec<_>>();
                    let wp = compute_cm(&weighted_points);
                    *cm = Some(wp);
                    wp
                }
            },
        }
    }

    /// Approximate points for force calculation using Barnes-Hut algorithm, lazily computing CoMs of nodes
    fn approximate_points(
        &mut self,
        subject: &impl Point,
        theta: f64,
        results: &mut Vec<WeightedPoint>,
    ) {
        match self {
            Self::Empty { .. } => (),
            Self::External { data, .. } => results.extend(data.iter().map(|x| WeightedPoint {
                pos: x.point(),
                mass: 1.0,
            })),
            Self::Internal {
                boundary, children, ..
            } => {
                let w = boundary.perimeter() / 4.0;
                let d = na::distance(&subject.point(), &boundary.center());
                if w / d < theta {
                    results.push(self.center_of_mass());
                } else {
                    for c in children {
                        c.approximate_points(subject, theta, results);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::point;

    use crate::util::tests::make_rect;

    use super::*;

    #[test]
    fn test_compute_cm() {
        let points = vec![
            point![0.0, 0.0],
            point![2.0, 0.0],
            point![0.0, 2.0],
            point![2.0, 2.0],
        ];

        let cm = &compute_cm(&points);
        assert_eq!(
            cm.pos,
            point![1.0, 1.0],
            "Position should be the center of mass"
        );
        assert_eq!(cm.mass, 4.0, "Total mass should be the sum of all masses");
    }

    #[test]
    fn test_compute_cm_weighted() {
        let points = vec![
            WeightedPoint {
                pos: point![0.0, 0.0],
                mass: 1.0,
            },
            WeightedPoint {
                pos: point![2.0, 0.0],
                mass: 1.0,
            },
            WeightedPoint {
                pos: point![0.0, 2.0],
                mass: 1.0,
            },
            WeightedPoint {
                pos: point![2.0, 2.0],
                mass: 5.0,
            },
        ];

        let cm = &compute_cm(&points);
        assert_eq!(
            cm.pos,
            point![1.5, 1.5],
            "Position should be the center of mass"
        );
        assert_eq!(cm.mass, 8.0, "Total mass should be the sum of all masses");
    }

    #[test]
    fn barnes_hut() {
        let mut qt = QuadTree::new(make_rect(0.0, 0.0, 100.0, 100.0), 1);

        let points = vec![
            // 1st quadrant
            point![10.0, 10.0],
            point![10.0, 40.0],
            point![40.0, 10.0],
            point![40.0, 40.0],
            // 2nd quadrant
            point![10.0, 60.0],
            point![10.0, 90.0],
            point![40.0, 60.0],
            point![40.0, 90.0],
            // 3rd quadrant
            point![60.0, 10.0],
            point![60.0, 40.0],
            point![90.0, 10.0],
            point![90.0, 40.0],
            // 4th quadrant
            point![60.0, 60.0],
            point![60.0, 90.0],
            point![90.0, 60.0],
            point![90.0, 90.0],
        ];

        qt.insert_many(&points);

        let approx = qt.barnes_hut(&point![25.0, 25.0], 1.0);
        assert_eq!(approx.len(), 13, "Points were collapsed with theta 1.0");
        assert!(approx.contains(&WeightedPoint::new(point![75.0, 75.0], 4.0)));

        let approx = qt.barnes_hut(&point![25.0, 25.0], 2.0);
        assert_eq!(approx.len(), 7, "Points were collapsed with theta 2.0");
        assert!(approx.contains(&WeightedPoint::new(point![25.0, 75.0], 4.0)));
        assert!(approx.contains(&WeightedPoint::new(point![75.0, 25.0], 4.0)));

        let approx = qt.barnes_hut(&point![50.0, 50.0], 0.0);
        assert_eq!(approx.len(), 16, "All points were included with theta 0.0");

        let approx = qt.barnes_hut(&point![0.0, 0.0], 5.0);
        assert_eq!(
            approx.len(),
            1,
            "All points were collapsed with high theta and point in corner"
        );
        assert!(approx.contains(&WeightedPoint::new(point![50.0, 50.0], 16.0)));
    }
}
