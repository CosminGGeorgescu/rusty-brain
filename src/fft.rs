use core::f32;
use nalgebra::Complex;
use ndarray::{s, Array1, Array2, ArrayBase, Data, Ix1};
use num_traits::identities::Zero;
use std::f32::consts::PI;

// Trait which implements a different FFT algorithms, from complex-valued time-domain data to
// complex-valued frequency-domain
pub trait FourierTransform {
    fn dft(&self) -> Array1<Complex<f32>>;
    // Cooley-Tukey radix-2 algorithm
    fn fft(&self) -> Array1<Complex<f32>>;
}

// Trait which implements different FFT algorithms, from real-valued time-domain data to
// complex-valued frequency-domain
pub trait RealFourierTransform {
    // Naive DFT
    fn dft(&self) -> Array1<Complex<f32>>;
    // Cooley-Tukey radix-2 FFT algorithm
    // !!!!!!!!!!!
    // Does the same thing as normal FFT, just maps the signal to complex values
    fn rfft(&self) -> Array1<Complex<f32>>;
    // Short-time FT implementation using a sine window
    fn stft(&self, window_size: usize, hop_size: usize) -> Array2<Complex<f32>>;
}

// Trait which implements an inverse FFT algorithm, from complex-valued frequency-domain to
// complex-valued time-domain
pub trait InverseFourierTransform {
    fn idft(&self) -> Array1<Complex<f32>>;
    // Conjugate trick for computing the inverse FFT
    fn ifft(&self) -> Array1<Complex<f32>>;
}

// Trait which implements different inverse FFT algorithms, from complex-valued frequency-domain to
// real-vlaued time-domain
pub trait RealInverseFourierTransform {
    fn irdft(&self) -> Array1<f32>;
    // One way of computing the inverse FFT for a real-valued signal
    // Considers that the original signal was real-valued only
    fn irfft(&self) -> Array1<f32>;
}

impl<S> FourierTransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = Complex<f32>>,
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

    fn fft(&self) -> Array1<Complex<f32>> {
        let n = self.len();

        // Base case
        if n == 1 {
            return Array1::from_elem(1, self[0]);
        }

        // Divide the input signal into even and odd indices subslices
        let even = self.slice(s![..; 2]);
        let odd = self.slice(s![1..; 2]);

        // Recurse on the subslices
        let fft_even = even.fft();
        let fft_odd = odd.fft();

        // Return the full spectrum of frequencies
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
        self.mapv(|r| Complex::new(r, 0.0)).fft()
    }

    fn stft(&self, window_size: usize, hop_size: usize) -> Array2<Complex<f32>> {
        // Pad the window size to be of power-of-2 length
        let next_pow_2 = window_size.next_power_of_two();
        // Construct the sine window function
        let window = Array1::from_iter(
            (0..window_size).map(|n| (PI * (n as f32 + 0.5) / window_size as f32).sin()),
        );
        let num_frames = (self.len() - window_size) / hop_size + 1;
        let mut result = Array2::<Complex<f32>>::zeros((num_frames, next_pow_2));

        for i in 0..num_frames {
            let start = i * hop_size;
            let mut frame = Array1::zeros(next_pow_2);
            frame
                .slice_mut(s![..window_size])
                .assign(&(&self.slice(s![start..start + window_size]) * &window));
            let spectrum = frame.rfft();

            result.slice_mut(s![i, ..]).assign(&spectrum);
        }

        result
    }
}

impl<S> InverseFourierTransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = Complex<f32>>,
{
    fn idft(&self) -> Array1<Complex<f32>> {
        let n = self.len();
        let mut result = Array1::zeros(n);

        for k in 0..n {
            let mut sum = Complex::zero();

            for t in 0..n {
                let angle = 2.0f32 * PI * k as f32 * t as f32 / n as f32;
                let twiddle = Complex::new(angle.cos(), angle.sin());

                sum += twiddle * self[t];
            }

            result[k] = sum;
        }

        result / n as f32
    }

    fn ifft(&self) -> Array1<Complex<f32>> {
        // Conjugates the data, applies a forward FFT then conjugates the output back
        self.map(|x| x.conj()).fft().map(|x| x.conj()) / self.len() as f32
    }
}

impl<S> RealInverseFourierTransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = Complex<f32>>,
{
    fn irdft(&self) -> Array1<f32> {
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
        // Conjugates the data, applies a forward FFT, conjugates the output and discards the imaginary
        // part
        self.mapv(Complex::from).ifft().map(|z| z.re)
    }
}

// Computes the FFT frequencies for an n-point FFT with the `sampling_freq` in Hz
pub fn freqs(n: usize, sampling_freq: f32) -> Array1<f32> {
    let df = sampling_freq / n as f32;
    Array1::from_iter((0..n).map(|i| (i - if i < n / 2 { 0 } else { n }) as f32 * df))
}

// Computes the FFT frequencies for an n-point real FFT with the `sampling_freq` in Hz
pub fn rfreqs(n: usize, sampling_freq: f32) -> Array1<f32> {
    let n_pos = n / 2 + 1;
    Array1::from_iter((0..n_pos).map(|i| i as f32 * sampling_freq / n as f32))
}
