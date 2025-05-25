pub mod covariance;
pub mod fft;
pub mod filter;
#[allow(dead_code)]
pub mod read;
pub mod s_transform;

#[cfg(test)]
mod tests {
    mod filter {
        use approx::assert_abs_diff_eq;
        use ndarray::Array1;

        use crate::filter::FIRFilter;

        #[test]
        fn impulse_response() {
            let coeffs = vec![0.5, 0.25, 0.25];
            let filter = FIRFilter::new(coeffs.clone());

            let mut impulse = Array1::zeros(1);
            impulse[0] = 1.0;

            let output = filter.process(&impulse);

            for (o, c) in output.iter().zip(coeffs.iter()) {
                assert_abs_diff_eq!(o, c, epsilon = 1e-5);
            }
        }

        #[test]
        fn zero_signal() {
            let coeffs = vec![0.2, 0.3, 0.5];
            let filter = FIRFilter::new(coeffs);

            let signal = Array1::zeros(10);
            let output = filter.process(&signal);

            for o in output.iter() {
                assert_abs_diff_eq!(*o, 0.0, epsilon = 1e-5);
            }
        }

        #[test]
        fn moving_average() {
            let coeffs = vec![1.0 / 3.0; 3];
            let filter = FIRFilter::new(coeffs);

            let signal = Array1::from(vec![3.0, 6.0, 9.0, 12.0, 15.0]);
            let output = filter.process(&signal);

            let expected = [
                1.0,
                (3.0 + 6.0) / 3.0,
                (3.0 + 6.0 + 9.0) / 3.0,
                (6.0 + 9.0 + 12.0) / 3.0,
                (9.0 + 12.0 + 15.0) / 3.0,
                (12.0 + 15.0) / 3.0,
                15.0 / 3.0,
            ];

            for (o, e) in output.iter().zip(expected.iter()) {
                assert_abs_diff_eq!(o, e, epsilon = 1e-5);
            }
        }
        #[test]
        fn short_signal() {
            let coeffs = vec![0.5, 0.5];
            let filter = FIRFilter::new(coeffs);

            let signal = Array1::from(vec![1.0]);
            let output = filter.process(&signal);

            let expected = [0.5, 0.5];

            for (o, e) in output.iter().zip(expected.iter()) {
                assert_abs_diff_eq!(o, e, epsilon = 1e-5);
            }
        }
    }
}
