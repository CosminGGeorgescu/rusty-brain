// R. G. Stockwell, L. Mansinha and R. P. Lowe, "Localization of the complex spectrum: the S transform," in IEEE Transactions on Signal Processing, vol. 44, no. 4, pp. 998-1001, April 1996, doi: 10.1109/78.492555.

use std::f32::consts::PI;

use nalgebra::Complex;
use ndarray::{Array1, Array2, ArrayBase, Data, Ix1, Ix2};

use crate::fft::{FourierTransform, InverseFourierTransform};

pub trait STransform {
    // Stockwell Transform
    // Computations are done in the Fourier Transform form
    // Should only be applied on signals of length power of 2 for compatibility with the FFT implementation
    fn st(&self) -> Array2<Complex<f32>>;
}

pub trait InverseSTransform {
    // Inverse Stockwell Transform
    fn ist(&self) -> Array1<f32>;
}

impl<S> STransform for ArrayBase<S, Ix1>
where
    S: Data<Elem = f32>,
{
    #[allow(non_snake_case)]
    fn st(&self) -> Array2<Complex<f32>> {
        let n = self.len();
        let gauss = |n: usize, m: usize| (-2.0 * PI * PI * (m * m) as f32 / (n * n) as f32).exp();

        // Compute FFT of signal
        let H = self.map(Complex::from).fft();

        let mut result = Array2::<Complex<f32>>::zeros((n / 2 + 1, n));

        result.row_mut(0).assign(&Array1::from_elem(
            n,
            Complex::new(self.mean().unwrap(), 0.0),
        ));
        for f in 1..=n / 2 {
            // Build Gaussian in frequency domain
            let mut wgauss = Array1::zeros(n);
            wgauss[0] = gauss(f, 0);
            for i in 1..=n / 2 {
                let val = gauss(f, i);
                wgauss[i] = val;
                wgauss[n - i] = val;
            }

            // Multiply FFT{x} by frequency-localized Gaussian
            // Correct by Convolution Theorem
            let filtered = Array1::from_shape_fn(n, |i| {
                let mut k = i + f;
                if k >= n {
                    k -= n;
                }
                H[k] * wgauss[i]
            });

            // Compute Inverse FFT to get back time-localized signal
            let inverse = filtered.ifft();

            result.row_mut(f).assign(&inverse);
        }

        result
    }
}

impl<S> InverseSTransform for ArrayBase<S, Ix2>
where
    S: Data<Elem = Complex<f32>>,
{
    fn ist(&self) -> Array1<f32> {
        let (nfreqs, ntimes) = self.dim();

        let mut result = Array1::<Complex<f32>>::zeros(ntimes);

        // Sum each frequency component across time
        for f in 0..nfreqs {
            result[f] = self.row(f).sum();
        }

        for i in ntimes / 2 + 1..ntimes {
            result[i] = result[ntimes - i].conj();
        }

        result.ifft().map(|z| z.re)
    }
}
