use wasm_bindgen::prelude::*;

use crate::{bowyer_watson, Point2D};

#[wasm_bindgen]
pub fn triangulate(coords: &[f64]) -> Result<JsValue, JsError> {
    if coords.len() % 2 != 0 {
        return Err(JsError::new(
            "coords must have an even number of elements (x1,y1,x2,y2,...)",
        ));
    }

    let points: Vec<Point2D> = coords
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| Point2D {
            x: chunk[0],
            y: chunk[1],
            index: i as i64,
        })
        .collect();

    let triangles = bowyer_watson(points);

    let result: Vec<[usize; 3]> = triangles
        .iter()
        .map(|t| [t.a.index as usize, t.b.index as usize, t.c.index as usize])
        .collect();

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}
