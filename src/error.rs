use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeshingError {
    #[error("input points vector is empty")]
    EmptyInput,
    #[error("insufficient points for triangulation: need at least 3, got {0}")]
    InsufficientPoints(usize),
}
