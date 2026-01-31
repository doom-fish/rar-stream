//! PPMd (Prediction by Partial Matching, variant D) decompression.
//!
//! This implements the PPMd algorithm used in RAR3/RAR4 archives.
//! Based on Dmitry Shkarin's PPMd implementation.

mod range_coder;
mod sub_alloc;
mod model;

pub use model::PpmModel;
pub use range_coder::RangeCoder;
