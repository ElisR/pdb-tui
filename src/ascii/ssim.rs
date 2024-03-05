//! Calculating SSIM metrics used for glyph lookup
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SsimError {
    #[error("Input slices must have the same length.")]
    UnequalLengths,
}
// Constants for SSIM calculation
const K1: f32 = 0.01;
const K2: f32 = 0.03;
const L: f32 = 1.0; // Dynamic range

/// Return the SSIM for test and references
pub fn ssim(test: &[f32], reference: &[f32]) -> Result<f32, SsimError> {
    if test.len() != reference.len() {
        return Err(SsimError::UnequalLengths);
    }

    let mu_x = test.iter().sum::<f32>() / test.len() as f32;
    let mu_y = reference.iter().sum::<f32>() / reference.len() as f32;

    let sigma_x = (test.iter().map(|&i| i.powi(2)).sum::<f32>() / test.len() as f32).sqrt();
    let sigma_y =
        (reference.iter().map(|&i| i.powi(2)).sum::<f32>() / reference.len() as f32).sqrt();

    let sigma_xy = test
        .iter()
        .zip(reference.iter())
        .map(|(&x, &y)| x * y)
        .sum::<f32>()
        / test.len() as f32
        - mu_x * mu_y;

    let c1 = (K1 * L).powi(2);
    let c2 = (K2 * L).powi(2);

    let ssim = (2.0 * mu_x * mu_y + c1) * (2.0 * sigma_xy + c2)
        / ((mu_x.powi(2) + mu_y.powi(2) + c1) * (sigma_x.powi(2) + sigma_y.powi(2) + c2));

    Ok(ssim)
}
