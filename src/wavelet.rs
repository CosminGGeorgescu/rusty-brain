use nalgebra::Complex;
use ndarray::Array1;
use ndarray::Array2;
use ndarray::ArrayBase;
use ndarray::Data;
use ndarray::Ix1;
use num_traits::Float;
use num_traits::Zero;

use crate::fft::RealFourierTransform;

pub trait Wavelet {
    type Dtype: Float;
    type WaveletDtype: Into<Complex<f32>>;

    fn generate(time: &Array1<Self::Dtype>, omega: Self::Dtype) -> Array1<Self::WaveletDtype>;

    fn generate_inplace(time: &mut Array1<Self::WaveletDtype>, omega: Self::Dtype);
}

pub struct Morlet;

impl Wavelet for Morlet {
    type Dtype = f32;
    type WaveletDtype = Complex<f32>;

    fn generate(time: &Array1<f32>, omega: f32) -> Array1<Complex<f32>> {
        time.map(|&t| {
            let gaussian = (-0.5f32 * t * t).exp();
            let sinusoid = Complex::new(0.0f32, omega * t).exp();

            sinusoid * gaussian
        })
    }

    fn generate_inplace(time: &mut Array1<Complex<f32>>, omega: f32) {
        for t in time.iter_mut() {
            let t_re = t.re;
            let gaussian = (-0.5f32 * t_re * t_re).exp();
            let sinusoid = Complex::new(0.0, omega * t_re).exp();

            *t = sinusoid * gaussian;
        }
    }
}

pub struct MexicanHat;

impl Wavelet for MexicanHat {
    type Dtype = f32;
    type WaveletDtype = f32;

    fn generate(time: &Array1<f32>, omega: f32) -> Array1<f32> {
        time.map(|&t| {
            let normalized_time = (t / omega).powi(2);

            let factor = 1.0 - normalized_time;
            let gaussian = (-0.5 * normalized_time).exp();

            factor * gaussian
        })
    }

    fn generate_inplace(time: &mut Array1<f32>, omega: f32) {
        for t in time.iter_mut() {
            let normalized_time = (*t / omega).powi(2);

            let factor = 1.0 - normalized_time;
            let gaussian = (-0.5 * normalized_time).exp();

            *t = factor * gaussian;
        }
    }
}

pub trait WaveletTransform {
    fn cwt<T>(&self, scale: &[f32]) -> Array2<Complex<f32>>
    where
        T: Wavelet<Dtype = f32>,
        T::WaveletDtype: Into<Complex<f32>> + Clone;
}

impl<S> WaveletTransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = f32>,
{
    fn cwt<T>(&self, scales: &[f32]) -> Array2<Complex<f32>>
    where
        T: Wavelet<Dtype = f32>,
        T::WaveletDtype: Into<Complex<f32>> + Clone,
    {
        let n = self.len();
        let times = Array1::from_iter(0..n);

        let mut result = Array2::zeros((scales.len(), n));

        for (i, a) in scales.iter().enumerate() {
            let normalization_factor = 1.0 / a.sqrt();

            let mut scale_result = Array1::zeros(n);

            for &b in times.iter() {
                let shifted_scaled_time = times.map(|t| (t - b) as f32 / a);

                let wavelet_coeffs_conj =
                    T::generate(&shifted_scaled_time, 6.0).mapv(|v| v.into().conj());

                let coeff: Complex<f32> = self
                    .iter()
                    .zip(wavelet_coeffs_conj)
                    .map(|(x, w)| x * w)
                    .sum();

                scale_result[b] = normalization_factor * coeff;
            }

            result.row_mut(i).assign(&scale_result);
        }

        result
    }
}
