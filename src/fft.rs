use nalgebra::Complex;
use ndarray::s;
use ndarray::ArrayBase;
use ndarray::Data;
use ndarray::Ix1;
use ndarray::{Array1, Array2};
use num_traits::identities::Zero;
use std::f32::consts::PI;

pub trait FourierTransform<S>
where
    S: Data<Elem = f32>,
{
    fn dft(&self) -> Array1<Complex<f32>>;
    fn fft(&self) -> Array1<Complex<f32>>;
    fn stft(&self, window_size: usize, hop_size: usize) -> Array2<f32>;
}

impl<S> FourierTransform<S> for ArrayBase<S, Ix1>
where
    S: Data<Elem = f32>,
{
    fn dft(&self) -> Array1<Complex<f32>> {
        let n = self.len();
        let mut result = Array1::zeros(n);

        for k in 0..n {
            let mut sum = Complex::zero();

            for t in 0..n {
                let angle = -2.0f32 * std::f32::consts::PI * k as f32 * t as f32 / n as f32;
                let twiddle = Complex::new(angle.cos(), angle.sin());

                sum += twiddle * self[t];
            }

            result[k] = sum;
        }

        result
    }

    fn fft(&self) -> Array1<Complex<f32>> {
        let n = self.len();

        if n == 1 {
            return Array1::from_elem(1, Complex::from(self[0]));
        }

        let even = self.slice(s![..; 2]);
        let odd = self.slice(s![1..; 2]);

        let fft_even = even.fft();
        let fft_odd = odd.fft();

        let mut result = Array1::zeros(n);
        for k in 0..n / 2 {
            let angle = -2.0 * std::f32::consts::PI * k as f32 / n as f32;
            let twiddle = Complex::new(angle.cos(), angle.sin());

            result[k] = fft_even[k] + twiddle * fft_odd[k];
            result[k + n / 2] = fft_even[k] - twiddle * fft_odd[k];
        }

        result
    }

    fn stft(&self, window_size: usize, hop_size: usize) -> Array2<f32> {
        let window = Array1::from_iter(
            (0..window_size).map(|n| (PI * (n as f32 + 0.5) / window_size as f32).sin()),
        );
        let num_frames = (self.len() - window_size) / hop_size + 1;
        let mut result = Array2::<f32>::zeros((num_frames, window_size));

        for i in 0..num_frames {
            let start = i * hop_size;
            let frame =
                &Array1::from(self.slice(s![start..start + window_size]).to_owned()) * &window;
            let spectrum = frame.view().fft();

            result
                .slice_mut(s![i, ..])
                .assign(&spectrum.map(|c| c.norm()));
        }

        result
    }
}
