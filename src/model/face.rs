use crate::model::point_3d::Point3D;

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub a: Point3D,
    pub b: Point3D,
    pub c: Point3D,
}

impl Face {
    pub fn vertices(&self) -> [Point3D; 3] {
        [self.a, self.b, self.c]
    }
}

impl PartialEq for Face {
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
