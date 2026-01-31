//! PPMd (Prediction by Partial Matching, variant D) decompression.
//!
//! This implements the PPMd algorithm used in RAR3/RAR4 archives.
//! Based on Dmitry Shkarin's PPMd implementation.

mod model;
mod range_coder;
mod sub_alloc;

pub use model::PpmModel;
pub use range_coder::RangeCoder;
