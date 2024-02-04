use crate::{Edge, Point2D, Triangle};

pub fn create_super_triangle(points: &Vec<Point2D>) -> Triangle {
    match points.is_empty() {
        true => panic!("The input points vector should not be empty."),
        false => {}
    }

    let index = 0;
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for point in points {
        if point.x < min_x {
            min_x = point.x;
        }
        if point.y < min_y {
            min_y = point.y;
        }
        if point.x > max_x {
            max_x = point.x;
        }
        if point.y > max_y {
            max_y = point.y;
        }
    }

    let margin = 100.0;

    let a = Point2D {
        index,
        x: min_x - margin,
        y: min_y - margin,
    };
    let b = Point2D {
        index,
        x: max_x + margin,
        y: min_y - margin,
    };
    let c = Point2D {
        index,
        x: (min_x + max_x) / 2.0,
        y: max_y + margin,
    };

    Triangle { a, b, c }
}

pub fn edge_is_shared_by_triangles(edge: &Edge, triangles: &Vec<Triangle>) -> bool {
    for triangle in triangles {
        let edges_of_triangle = triangle.edges();
        for edge_of_triangle in edges_of_triangle {
            if edge_of_triangle == *edge {
                return true;
            }
            if edge_of_triangle.reverse() == *edge {
                return true;
            }
        }
    }
    false
}

pub fn retriangulate(edge: &Edge, point: &Point2D) -> Triangle {
    Triangle {
        a: edge.start,
        b: edge.end,
        c: *point,
    }
}
