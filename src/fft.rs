use nalgebra::Complex;
use ndarray::s;
use ndarray::ArrayBase;
use ndarray::Data;
use ndarray::Ix1;
use ndarray::{Array1, Array2};
use num_traits::identities::Zero;
use std::f32::consts::PI;

// Trait which implements a single FFT algorithm on real-valued time-domain data
// Used in computing the inverse FFT
trait FourierTransform {
    // Cooley-Tukey radix-2 FFT algorithm
    fn fft(&self) -> Array1<Complex<f32>>;
}

// Trait which implements different FFT algorithms on real-valued time-domain data
pub trait RealFourierTransform {
    // Naive DFT
    fn dft(&self) -> Array1<Complex<f32>>;
    // Cooley-Tukey radix-2 FFT algorithm
    fn rfft(&self) -> Array1<Complex<f32>>;
    // Short-time FT implementation using a sine window
    fn stft(&self, window_size: usize, hop_size: usize) -> Array2<f32>;
}

// Trait which implement different inverse FFT algorithms to real-valued time-domain data
pub trait RealInverseFourierTransform {
    fn idft(&self) -> Array1<f32>;
    fn irfft(&self) -> Array1<f32>;
}

impl<S> FourierTransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = Complex<f32>>,
{
    fn fft(&self) -> Array1<Complex<f32>> {
        let n = self.len();

        if n == 1 {
            return Array1::from_elem(1, self[0]);
        }

        let even = self.slice(s![..; 2]);
        let odd = self.slice(s![1..; 2]);

        let fft_even = even.fft();
        let fft_odd = odd.fft();

        let mut result = Array1::zeros(n);
        for k in 0..n / 2 {
            let angle = -2.0 * PI * k as f32 / n as f32;
            let twiddle = Complex::new(angle.cos(), angle.sin());

            result[k] = fft_even[k] + twiddle * fft_odd[k];
            result[k + n / 2] = fft_even[k] - twiddle * fft_odd[k];
        }

        result
    }
}

impl<S> RealFourierTransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = f32>,
{
    fn dft(&self) -> Array1<Complex<f32>> {
        let n = self.len();
        let mut result = Array1::zeros(n);

        for k in 0..n {
            let mut sum = Complex::zero();

            for t in 0..n {
                let angle = -2.0f32 * PI * k as f32 * t as f32 / n as f32;
                let twiddle = Complex::new(angle.cos(), angle.sin());

                sum += twiddle * self[t];
            }

            result[k] = sum;
        }

        result
    }

    fn rfft(&self) -> Array1<Complex<f32>> {
        let n = self.len();

        if n == 1 {
            return Array1::from_elem(1, Complex::from(self[0]));
        }

        let even = self.slice(s![..; 2]);
        let odd = self.slice(s![1..; 2]);

        let fft_even = even.rfft();
        let fft_odd = odd.rfft();

        let mut result = Array1::zeros(n);
        for k in 0..n / 2 {
            let angle = -2.0 * PI * k as f32 / n as f32;
            let twiddle = Complex::new(angle.cos(), angle.sin());

            let ttodd = twiddle * fft_odd[k];
            result[k] = fft_even[k] + ttodd;
            result[k + n / 2] = fft_even[k] - ttodd;
        }

        result
    }

    fn stft(&self, window_size: usize, hop_size: usize) -> Array2<f32> {
        let next_pow_2 = window_size.next_power_of_two();
        let window = Array1::from_iter(
            (0..window_size).map(|n| (PI * (n as f32 + 0.5) / window_size as f32).sin()),
        );
        let num_frames = (self.len() - window_size) / hop_size + 1;
        let mut result = Array2::<f32>::zeros((num_frames, next_pow_2));

        for i in 0..num_frames {
            let start = i * hop_size;
            let mut frame = Array1::zeros(next_pow_2);
            frame
                .slice_mut(s![..window_size])
                .assign(&(&self.slice(s![start..start + window_size]) * &window));
            let spectrum = frame.rfft();

            result
                .slice_mut(s![i, ..])
                .assign(&spectrum.map(|c| c.norm()));
        }

        result
    }
}

impl<S> RealInverseFourierTransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = Complex<f32>>,
{
    fn idft(&self) -> Array1<f32> {
        let n = self.len();
        let mut result = Array1::zeros(n);

        for k in 0..n {
            let mut sum = 0.0f32;

            for t in 0..n {
                let angle = 2.0f32 * PI * k as f32 * t as f32 / n as f32;
                let twiddle = Complex::new(angle.cos(), angle.sin());

                sum += (twiddle * self[t]).re;
            }

            result[k] = sum;
        }

        result / n as f32
    }

    fn irfft(&self) -> Array1<f32> {
        self.map(|x| x.conj()).fft().map(|x| x.conj().norm()) / self.len() as f32
    }
}
