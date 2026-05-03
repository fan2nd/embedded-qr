#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

mod builder;
pub mod capacity;
mod ecc;
#[cfg(feature = "embedded-graphics")]
#[cfg_attr(docsrs, doc(cfg(feature = "embedded-graphics")))]
mod eg;
mod encoder;
mod helper;
mod interleave;
mod matrix;
mod matrix_template;
mod types;
mod version;

pub use builder::QrBuilder;
#[cfg(feature = "embedded-graphics")]
#[cfg_attr(docsrs, doc(cfg(feature = "embedded-graphics")))]
pub use eg::QrDrawable;
pub use matrix::{QrMatrix, QrMatrixIter};
pub use types::{EccLevel, EncodeMode, Mask, QrError};
pub use version::{
    Version, Version1, Version2, Version3, Version4, Version5, Version6, Version7, Version8,
    Version9, Version10, Version11, Version12, Version13, Version14, Version15, Version16,
    Version17, Version18, Version19, Version20, Version21, Version22, Version23, Version24,
    Version25, Version26, Version27, Version28, Version29, Version30, Version31, Version32,
    Version33, Version34, Version35, Version36, Version37, Version38, Version39, Version40,
};
