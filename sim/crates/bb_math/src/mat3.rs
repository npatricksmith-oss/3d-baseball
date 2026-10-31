//! 3×3 matrix, row-major. Used for inertia tensors and the collision K matrix.

use core::ops::{Add, Mul, Neg, Sub};

use crate::Vec3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mat3 {
    pub r: [[f64; 3]; 3],
}

impl Mat3 {
    pub const ZERO: Mat3 = Mat3 { r: [[0.0; 3]; 3] };
    pub const IDENTITY: Mat3 = Mat3 {
        r: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    pub const fn from_rows(r: [[f64; 3]; 3]) -> Mat3 {
        Mat3 { r }
    }
    pub fn diag(a: f64, b: f64, c: f64) -> Mat3 {
        Mat3::from_rows([[a, 0.0, 0.0], [0.0, b, 0.0], [0.0, 0.0, c]])
    }
    pub fn scale(s: f64) -> Mat3 {
        Mat3::diag(s, s, s)
    }
    /// Outer product a·bᵀ.
    pub fn outer(a: Vec3, b: Vec3) -> Mat3 {
        let a = a.to_array();
        let b = b.to_array();
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] = a[i] * b[j];
            }
        }
        Mat3 { r }
    }
    /// Skew-symmetric cross-product matrix: `skew(a) * b == a.cross(b)`.
    pub fn skew(a: Vec3) -> Mat3 {
        Mat3::from_rows([[0.0, -a.z, a.y], [a.z, 0.0, -a.x], [-a.y, a.x, 0.0]])
    }
    pub fn transpose(&self) -> Mat3 {
        let mut r = [[0.0; 3]; 3];
        for (i, row) in r.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v = self.r[j][i];
            }
        }
        Mat3 { r }
    }
    pub fn det(&self) -> f64 {
        let m = &self.r;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }
    /// Inverse by cofactors; `None` if singular.
    pub fn inverse(&self) -> Option<Mat3> {
        let d = self.det();
        if d == 0.0 || !d.is_finite() {
            return None;
        }
        let m = &self.r;
        let adj = [
            [
                m[1][1] * m[2][2] - m[1][2] * m[2][1],
                m[0][2] * m[2][1] - m[0][1] * m[2][2],
                m[0][1] * m[1][2] - m[0][2] * m[1][1],
            ],
            [
                m[1][2] * m[2][0] - m[1][0] * m[2][2],
                m[0][0] * m[2][2] - m[0][2] * m[2][0],
                m[0][2] * m[1][0] - m[0][0] * m[1][2],
            ],
            [
                m[1][0] * m[2][1] - m[1][1] * m[2][0],
                m[0][1] * m[2][0] - m[0][0] * m[2][1],
                m[0][0] * m[1][1] - m[0][1] * m[1][0],
            ],
        ];
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] = adj[i][j] / d;
            }
        }
        Some(Mat3 { r })
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        let m = &self.r;
        Vec3::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
        )
    }
}
impl Mul for Mat3 {
    type Output = Mat3;
    fn mul(self, o: Mat3) -> Mat3 {
        let mut r = [[0.0; 3]; 3];
        for (i, row) in r.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v = self.r[i][0] * o.r[0][j] + self.r[i][1] * o.r[1][j] + self.r[i][2] * o.r[2][j];
            }
        }
        Mat3 { r }
    }
}
impl Mul<f64> for Mat3 {
    type Output = Mat3;
    fn mul(self, s: f64) -> Mat3 {
        let mut r = self.r;
        for row in r.iter_mut() {
            for v in row.iter_mut() {
                *v *= s;
            }
        }
        Mat3 { r }
    }
}
impl Add for Mat3 {
    type Output = Mat3;
    fn add(self, o: Mat3) -> Mat3 {
        let mut r = self.r;
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] += o.r[i][j];
            }
        }
        Mat3 { r }
    }
}
impl Sub for Mat3 {
    type Output = Mat3;
    fn sub(self, o: Mat3) -> Mat3 {
        self + (-o)
    }
}
impl Neg for Mat3 {
    type Output = Mat3;
    fn neg(self) -> Mat3 {
        self * -1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skew_matches_cross() {
        let a = Vec3::new(0.3, -1.2, 2.0);
        let b = Vec3::new(-0.7, 0.4, 1.1);
        assert_eq!(Mat3::skew(a) * b, a.cross(b));
    }

    #[test]
    fn inverse_roundtrip() {
        let a = Mat3::from_rows([[4.0, 1.0, 0.5], [1.0, 3.0, -0.2], [0.5, -0.2, 2.0]]);
        let p = a * a.inverse().unwrap();
        for i in 0..3 {
            for j in 0..3 {
                let e = if i == j { 1.0 } else { 0.0 };
                assert!((p.r[i][j] - e).abs() < 1e-14);
            }
        }
    }
}
