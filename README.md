# Features

### Fourier Transforms
- Forward
	- naive DFT
	- Fast Fourier Transform using the Cooley-Tukey radix-2 algorithm
	- Short-time Fourier Transform using a sine window
- Inverse
	- naive IDFT# Features

### Fourier Transforms
- Forward
	- naive DFT
	- Fast Fourier Transform using the Cooley-Tukey radix-2 algorithm
	- Short-time Fourier Transform using a sine window
- Inverse
	- naive IDFT
	- [ ] IFFT
	- [ ] ISTFT

### Covariance
- Population and Sample covariance for 2-dimensional arrays
- Data orientation considered: $$N_{channels}\texttimes M_{samples}$$

### Loading data in [BrainVision Core Data Format 1.0](https://www.brainproducts.com/support-resources/brainvision-core-data-format-1-0/)

# Externals

### Eigendecomposition
- For Hermitian matrices, using `ndarray_linalg::eigh`

# Interesting datasets to be used
- https://doi.org/10.18112/openneuro.ds004264.v1.1.0
- https://doi.org/10.18112/openneuro.ds004951.v1.0.0
- https://doi.org/10.18112/openneuro.ds005520.v1.0.0
