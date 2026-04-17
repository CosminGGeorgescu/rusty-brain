# Features

### Fourier Transforms

Provides traits which are to be `impl`'d by structures on which the Fourier transform can be gracefully applied.

- Forward
	- naive DFT
	- Fast Fourier Transform using the Cooley-Tukey radix-2 algorithm
	- Short-time Fourier Transform using a sine window
- Inverse
	- naive IDFT
    - IFFT

Both forward and inverse traits are also split between:
- Normal FT: algorithms operating on complex-valued time-domain data
- Real FT: algorithms operating on real-valued time-domain data

and provides implementations for some convenient general structures:
- [ArrayBase<_, Ix1>](https://docs.rs/ndarray/0.16.0/ndarray/struct.ArrayBase.html)

and two util functions:
- freqs: FFT frequencies
- rfreqs: real FFT frequencies

### Stockwell Transforms

Provides the `STransform` and `InverseSTransform` traits which is to be `impl`'d by structures on which the Stockwell transform can be gracefully applied.

### Wavelet Transform

Provides the `WaveletTransform` traits which is to be `impl`'d by structures on which a Wavelet Transform of the following type can be gracefully applied:
- `cwt`: Continuous Wavelet Transform

### Covariance computation

- Population and Sample covariance for 2-dimensional arrays
- Data orientation considered: $$N_{channels}\texttimes M_{samples}$$

### Filtering
- FIR filtering using:
    - Overlap-Add method by FFT multiplications

### Wavelets

Provides the following wavelet structure (empty):
- Morlet
- Mexican Hat

along with the `Wavelet` trait which is to be implemented by structures that mimick a wavelet.

### Loading data
- Formats supported
	- [BrainVision Core Data Format 1.0](https://www.brainproducts.com/support-resources/brainvision-core-data-format-1-0/)

## Interesting datasets
- https://doi.org/10.18112/openneuro.ds004264.v1.1.0
- https://doi.org/10.18112/openneuro.ds004951.v1.0.0
- https://doi.org/10.18112/openneuro.ds005520.v1.0.0
