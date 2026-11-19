#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    pub index: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn distance_squared(&self, other: &Point3D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn distance(&self, p: &Point3D) -> f64 {
        self.distance_squared(p).sqrt()
    }
}
