use crate::model::point_3d::Point3D;

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Point3D,
    pub radius: f64,
}

impl Sphere {
    pub fn point_in_sphere(&self, point: &Point3D) -> bool {
        let squared_distance = self.center.distance_squared(point);
        squared_distance <= self.radius * self.radius
    }
}
