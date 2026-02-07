use crate::model::face::Face;
use crate::model::point_3d::Point3D;
use crate::model::sphere::Sphere;

#[derive(Debug, Clone, Copy)]
pub struct Tetrahedron {
    pub a: Point3D,
    pub b: Point3D,
    pub c: Point3D,
    pub d: Point3D,
}

impl Tetrahedron {
    pub fn circumsphere(&self) -> Sphere {
        let ax = self.a.x;
        let ay = self.a.y;
        let az = self.a.z;

        // Translate so that a is at origin
        let bx = self.b.x - ax;
        let by = self.b.y - ay;
        let bz = self.b.z - az;
        let cx = self.c.x - ax;
        let cy = self.c.y - ay;
        let cz = self.c.z - az;
        let dx = self.d.x - ax;
        let dy = self.d.y - ay;
        let dz = self.d.z - az;

        let b_sq = bx * bx + by * by + bz * bz;
        let c_sq = cx * cx + cy * cy + cz * cz;
        let d_sq = dx * dx + dy * dy + dz * dz;

        let det = bx * (cy * dz - cz * dy) - by * (cx * dz - cz * dx) + bz * (cx * dy - cy * dx);
        let inv_det = 1.0 / (2.0 * det);

        let ux = (b_sq * (cy * dz - cz * dy) - c_sq * (by * dz - bz * dy)
            + d_sq * (by * cz - bz * cy))
            * inv_det;
        let uy = -(b_sq * (cx * dz - cz * dx) - c_sq * (bx * dz - bz * dx)
            + d_sq * (bx * cz - bz * cx))
            * inv_det;
        let uz = (b_sq * (cx * dy - cy * dx) - c_sq * (bx * dy - by * dx)
            + d_sq * (bx * cy - by * cx))
            * inv_det;

        let center = Point3D {
            index: i64::MAX,
            x: ux + self.a.x,
            y: uy + self.a.y,
            z: uz + self.a.z,
        };
        let radius = center.distance(&self.a);

        Sphere { center, radius }
    }

    pub fn faces(&self) -> [Face; 4] {
        [
            Face {
                a: self.a,
                b: self.b,
                c: self.c,
            },
            Face {
                a: self.a,
                b: self.b,
                c: self.d,
            },
            Face {
                a: self.a,
                b: self.c,
                c: self.d,
            },
            Face {
                a: self.b,
                b: self.c,
                c: self.d,
            },
        ]
    }

    pub fn vertices(&self) -> [Point3D; 4] {
        [self.a, self.b, self.c, self.d]
    }

    pub fn signed_volume(&self) -> f64 {
        let ux = self.b.x - self.a.x;
        let uy = self.b.y - self.a.y;
        let uz = self.b.z - self.a.z;
        let vx = self.c.x - self.a.x;
        let vy = self.c.y - self.a.y;
        let vz = self.c.z - self.a.z;
        let wx = self.d.x - self.a.x;
        let wy = self.d.y - self.a.y;
        let wz = self.d.z - self.a.z;
        (ux * (vy * wz - vz * wy) - uy * (vx * wz - vz * wx) + uz * (vx * wy - vy * wx)) / 6.0
    }

    pub fn contains_face(&self, face: &Face) -> bool {
        let verts = self.vertices();
        verts.contains(&face.a) && verts.contains(&face.b) && verts.contains(&face.c)
    }
}

impl PartialEq for Tetrahedron {
    fn eq(&self, other: &Self) -> bool {
        let self_verts = self.vertices();
        let other_verts = other.vertices();
        for v in &other_verts {
            if !self_verts.contains(v) {
                return false;
            }
        }
        true
    }
}
