use nalgebra::Complex;
use ndarray::s;
use ndarray::Array1;
use ndarray::ArrayView1;
use std::f32::consts::PI;

pub trait FastFourierTransform<T> {
    fn fft(&self) -> Array1<Complex<T>>;
}

macro_rules! impl_fft_for {
    // Special treatment for `i16` vectors, required for full compatibility with
    (i16) => {
        impl<'a> FastFourierTransform<i16> for ArrayView1<'a, i16> {
            fn fft(&self) -> Array1<Complex<i16>> {
                let n = self.len();

                if n == 1 {
                    return Array1::from_elem(1, Complex::new(self[0], 0));
                }

                let even = self.slice(s![..; 2]);
                let odd = self.slice(s![1..; 2]);

                let fft_even = even.fft();
                let fft_odd = odd.fft();

                let mut result = Array1::zeros(n);
                for k in 0..n / 2 {
                    let angle = 2.0 * PI * k as f32 / n as f32;
                    let twiddle = Complex::new(
                        (angle.cos() * 32767.0_f32).round() as i16,
                        (angle.sin() * 32767.0_f32).round() as i16,
                    );

                    result[k] = fft_even[k] + fft_odd[k] * twiddle;
                    result[k + n / 2] = fft_even[k] - fft_odd[k] * twiddle;
                }

                result
            }
        }
    };
    // Funny special case for full compatibility with BrainVision Core Data Format 1.0
    // Implement FFT for floating-point types
    ($float_t: ty) => {
        impl<'a> FastFourierTransform<$float_t> for ArrayView1<'a, $float_t> {
            fn fft(&self) -> Array1<Complex<$float_t>> {
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
                    let angle = 2.0 * PI as $ float_t * k as $float_t / n as $float_t;
                    let twiddle = Complex::new(angle.cos(), angle.sin());
                    result[k] = fft_even[k] + twiddle * fft_odd[k];
                    result[k + n / 2] = fft_even[k] - twiddle * fft_odd[k];
                }

                result
            }
        }
    };
}

impl_fft_for!(i16);
impl_fft_for!(f32);
