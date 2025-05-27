use ndarray::{Array2, ArrayBase, Axis, Data, Ix2};

pub enum CovarianceType {
    Population = 0,
    Sample = 1,
}

pub trait Covariance<S>
where
    S: Data<Elem = f32>,
{
    // TODO
    // - Add `is_centered` parameter
    fn compute_covariance(&self, cov_t: CovarianceType) -> Array2<f32>;
}

impl<S> Covariance<S> for ArrayBase<S, Ix2>
where
    S: Data<Elem = f32>,
{
    fn compute_covariance(&self, cov_t: CovarianceType) -> Array2<f32> {
        let m_samples = self.dim().1;

        let mean = self.mean_axis(Axis(1)).unwrap().insert_axis(Axis(1));
        let centered = self - &mean;

        let covariance_matrix = centered.dot(&centered.t()) / (m_samples - cov_t as usize) as f32;

        covariance_matrix
    }
}
