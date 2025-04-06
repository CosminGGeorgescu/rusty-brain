use ndarray::{Array1, ArrayBase, Data, Ix1};

pub struct FIRFilter {
    coefficients: Array1<f32>,
    buffer: Array1<f32>,
}

impl FIRFilter {
    pub fn new<const N: usize>(coefficients: Vec<f32>) -> Self {
        Self {
            coefficients: coefficients.into(),
            buffer: vec![0.0f32; N].into(),
        }
    }
}
