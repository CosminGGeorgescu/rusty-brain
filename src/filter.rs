use nalgebra::Complex;
use ndarray::{s, Array1, ArrayBase, Data, Ix1};

use crate::fft::{RealFourierTransform, RealInverseFourierTransform};

pub struct FIRFilter {
    coefficients: Array1<f32>,
}

impl FIRFilter {
    pub fn new(coefficients: Vec<f32>) -> Self {
        Self {
            coefficients: coefficients.into(),
        }
    }

    #[allow(non_snake_case)]
    pub fn process<S>(&self, signal: &ArrayBase<S, Ix1>) -> Array1<f32>
    where
        S: Data<Elem = f32>,
    {
        let m = self.coefficients.len();
        let n = 8 * m.next_power_of_two();
        let l = n - m + 1;

        let mut padded_coef = Array1::zeros(n);
        padded_coef.slice_mut(s![..m]).assign(&self.coefficients);
        let H = padded_coef.rfft();

        let mut output = Array1::zeros(signal.len() + m - 1);

        for (i, chunk) in signal.axis_chunks_iter(ndarray::Axis(0), l).enumerate() {
            let mut padded_chunk = Array1::zeros(n);
            padded_chunk.slice_mut(s![..chunk.len()]).assign(&chunk);
            let X = padded_chunk.rfft();

            let y = X
                .iter()
                .zip(H.iter())
                .map(|(x, h)| x * h)
                .collect::<Array1<Complex<f32>>>()
                .irfft();

            let start = i * l;
            let end = (start + n).min(output.len());
            output
                .slice_mut(s![start..end])
                .iter_mut()
                .zip(y.iter())
                .for_each(|(o, v)| *o += v);
        }

        output
    }
}
