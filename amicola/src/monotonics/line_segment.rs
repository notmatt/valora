//! Line segments.

use super::*;
use crate::ext;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LineSegment {
    m: Slope,
    bounds: Bounds,
    start: V2,
    dir: V2,
    length: f32,
    normal: V2,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Slope {
    Vertical,
    Defined { m: f32, b: f32 },
}

impl LineSegment {
    pub fn new_rasterable(start: V2, end: V2) -> Option<Self> {
        if start.y == end.y {
            return None;
        }

        let (left, right) = ext::min_max_by(start, end, |p| p.x);
        let (bottom, top) = ext::min_max(start.y, end.y);
        let dx = right.x - left.x;
        let dy = right.y - left.y;
        let m = dy / dx;
        let m = if dx == 0.0 {
            Slope::Vertical
        } else {
            Slope::Defined {
                b: left.y - m * left.x,
                m: dy / dx,
            }
        };

        let dir = (end - start).normalize();
        let dir_theta = dir * std::f32::consts::PI / 2.;
        let normal = V2::new(
            dir_theta.x.cos() - dir_theta.y.sin(),
            dir_theta.x.sin() - dir_theta.y.cos(),
        )
        .normalize();

        Some(LineSegment {
            m,
            bounds: Bounds {
                left: left.x,
                right: right.x,
                top,
                bottom,
            },
            start,
            dir,
            length: (start - end).norm(),
            normal,
        })
    }
}

impl Curve for LineSegment {
    fn sample_y(&self, y: f32) -> Option<Intersection> {
        if self.bounds.bottom <= y && y <= self.bounds.top {
            match self.m {
                Slope::Vertical => Some(Intersection {
                    axis: self.bounds.right,
                    t: (y - self.bounds.bottom) / (self.bounds.top - self.bounds.bottom),
                }),
                Slope::Defined { m, b } => {
                    let x = (y - b) / m;
                    let p = V2::new(x, y);
                    Some(Intersection {
                        axis: x,
                        t: (p - self.start).norm() / self.length,
                    })
                }
            }
        } else {
            None
        }
    }

    fn sample_x(&self, x: f32) -> Option<Intersection> {
        if self.bounds.left <= x && x <= self.bounds.right {
            match self.m {
                Slope::Vertical => None,
                Slope::Defined { m, b } => {
                    let y = m * x + b;
                    let p = V2::new(x, y);
                    Some(Intersection {
                        axis: y,
                        t: (p - self.start).norm() / self.length,
                    })
                }
            }
        } else {
            None
        }
    }

    fn sample_t(&self, t: f32) -> Option<V2> {
        if t < 0.0 || t > 1.0 {
            return None;
        }

        Some(self.start + self.dir * t * self.length)
    }

    fn bounds(&self) -> &Bounds { &self.bounds }

    fn bookends(&self) -> (V2, V2) { (self.start, self.start + self.dir * self.length) }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn line_segment_new_valid() {
        assert_eq!(
            LineSegment::new_rasterable(V2::new(3.0, 1.0), V2::new(4.0, 2.0)),
            Some(LineSegment {
                m: Slope::Defined { m: 1.0, b: -2.0 },
                bounds: Bounds {
                    left: 3.0,
                    right: 4.0,
                    top: 2.0,
                    bottom: 1.0
                },
                start: V2::new(3.0, 1.0),
                dir: V2::new(0.70710677, 0.70710677),
                length: 1.4142135623730951,
                normal: V2::new(-0.7071068, 0.7071068)
            })
        );
    }

    #[test]
    fn line_segment_new_valid_steep_slope() {
        assert_eq!(
            LineSegment::new_rasterable(V2::new(3.0, 1.0), V2::new(4.0, 3.0)),
            Some(LineSegment {
                m: Slope::Defined { m: 2.0, b: -5.0 },
                bounds: Bounds {
                    left: 3.0,
                    right: 4.0,
                    top: 3.0,
                    bottom: 1.0
                },
                start: V2::new(3.0, 1.0),
                dir: V2::new(0.4472136, 0.8944272,),
                length: 2.23606797749979,
                normal: V2::new(-0.4206461, 0.90722483)
            })
        );
    }

    #[test]
    fn line_segment_new_invalid() {
        assert_eq!(
            LineSegment::new_rasterable(V2::new(3.0, 1.0), V2::new(6.0, 1.0)),
            None
        );
    }

    #[test]
    fn sample() {
        let segment =
            LineSegment::new_rasterable(V2::new(3.0, 1.0), V2::new(4.0, 2.0)).expect("segment");
        assert_eq!(
            segment.sample_y(1.0),
            Some(Intersection { axis: 3.0, t: 0.0 })
        );
        assert!(segment
            .sample_y(1.5)
            .map(|i| (i.axis - 3.5).abs() < 0.1)
            .unwrap_or(false));
        assert_eq!(
            segment.sample_y(2.0),
            Some(Intersection { axis: 4.0, t: 1.0 })
        );
        assert_eq!(segment.sample_y(2.1), None);
    }

    #[test]
    fn line_segment_new_triangle_edges() {
        assert_eq!(
            LineSegment::new_rasterable(V2::new(0.0, 0.0), V2::new(0.0, 100.0)),
            Some(LineSegment {
                m: Slope::Vertical,
                bounds: Bounds {
                    left: 0.0,
                    right: 0.0,
                    top: 100.0,
                    bottom: 0.0
                },
                start: V2::new(0.0, 0.0),
                dir: V2::new(0.0, 1.0),
                length: 100.0,
                normal: V2::new(0., 1.0)
            })
        );

        assert_eq!(
            LineSegment::new_rasterable(V2::new(0.0, 100.0), V2::new(100.0, 0.0)),
            Some(LineSegment {
                m: Slope::Defined { m: -1.0, b: 100.0 },
                bounds: Bounds {
                    left: 0.0,
                    right: 100.0,
                    top: 100.0,
                    bottom: 0.0
                },
                start: V2::new(0.0, 100.0),
                dir: V2::new(0.70710677, -0.70710677),
                length: 141.4213562373095,
                normal: V2::new(0.9475477, 0.31961447)
            })
        );

        assert_eq!(
            LineSegment::new_rasterable(V2::new(100.0, 0.0), V2::new(0.0, 0.0)),
            None
        );
    }
}