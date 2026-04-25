use crate::{
    ecc::VersionGeneratorTable,
    helper::{max_ecc_codewords_per_block, total_codewords},
    matrix_template::{VersionMatrixTemplates, build_version_matrix_templates},
    types::{DataBlocks, DatawordsCapacity, EccLevel},
};

#[doc(hidden)]
pub trait ZeroBuffer {
    fn zeroed() -> Self;
}

impl<const N: usize> ZeroBuffer for [u8; N] {
    fn zeroed() -> Self {
        [0; N]
    }
}

mod sealed {
    use super::{VersionMatrixTemplates, ZeroBuffer};

    pub trait SealedVersion: Sized {
        type DataEccBuf: AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug + ZeroBuffer;
        type InterleaveEccBuf: AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug + ZeroBuffer;
        type InterleaveVisitedBuf: AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug + ZeroBuffer;
        type MatrixBuf: AsRef<[u8]> + AsMut<[u8]> + Clone + core::fmt::Debug + ZeroBuffer + 'static;
        fn matrix_templates() -> &'static VersionMatrixTemplates<Self::MatrixBuf>;
    }
}

pub(crate) type DataEccBuf<T> = <T as sealed::SealedVersion>::DataEccBuf;
pub(crate) type InterleaveEccBuf<T> = <T as sealed::SealedVersion>::InterleaveEccBuf;
pub(crate) type InterleaveVisitedBuf<T> = <T as sealed::SealedVersion>::InterleaveVisitedBuf;
pub(crate) type MatrixBuf<T> = <T as sealed::SealedVersion>::MatrixBuf;

pub(crate) fn new_data_ecc_buf<T: Version>() -> DataEccBuf<T> {
    <DataEccBuf<T> as ZeroBuffer>::zeroed()
}

pub(crate) fn new_interleave_ecc_buf<T: Version>() -> InterleaveEccBuf<T> {
    <InterleaveEccBuf<T> as ZeroBuffer>::zeroed()
}

pub(crate) fn new_interleave_visited_buf<T: Version>() -> InterleaveVisitedBuf<T> {
    <InterleaveVisitedBuf<T> as ZeroBuffer>::zeroed()
}

pub(crate) fn matrix_templates<T: Version>() -> &'static VersionMatrixTemplates<MatrixBuf<T>> {
    <T as sealed::SealedVersion>::matrix_templates()
}

/// QR version metadata for a typed QR symbol.
///
/// This trait is implemented by the provided `Version1` through `Version40`
/// marker types. It is primarily useful as a generic bound for APIs that work
/// with a caller-selected QR version.
pub trait Version: sealed::SealedVersion + Sized {
    /// QR Code version number.
    const VERSION: usize;
    /// Side length of the matrix in modules.
    const WIDTH: usize = (Self::VERSION * 4) + 17;
    /// Total number of codewords in the symbol.
    const TOTAL_CODEWORDS: usize = total_codewords::<Self>();
    /// Data codeword capacity for each ECC level.
    const CAPACITY: DatawordsCapacity = DatawordsCapacity::get_from_version::<Self>();
    /// Block count for each ECC level.
    const BLOCKS: DataBlocks = DataBlocks::get_from_version::<Self>();
    /// Reed-Solomon generator table entries needed by this version.
    const RS_GENERATORS: VersionGeneratorTable = VersionGeneratorTable::build::<Self>();
}

macro_rules! define_versions {
    ($($name:ident => $number:expr),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default)]
            #[doc = concat!(
                "QR Code version ",
                stringify!($number),
                " marker type."
            )]
            pub struct $name;

            impl sealed::SealedVersion for $name {
                type DataEccBuf = [u8; Self::TOTAL_CODEWORDS];
                type InterleaveEccBuf = [u8; max_ecc_codewords_per_block::<Self>()];
                type InterleaveVisitedBuf =
                    [u8; (Self::CAPACITY.for_level(EccLevel::L) + 7) / 8];
                type MatrixBuf = [u8; (Self::WIDTH * Self::WIDTH + 7) / 8];

                fn matrix_templates() -> &'static VersionMatrixTemplates<Self::MatrixBuf> {
                    static MATRIX_TEMPLATES: VersionMatrixTemplates<
                        [u8; (((($number * 4) + 17) * (($number * 4) + 17) + 7) / 8)],
                    > = build_version_matrix_templates::<
                        $number,
                        { ($number * 4) + 17 },
                        { (((($number * 4) + 17) * (($number * 4) + 17)) + 7) / 8 },
                    >();
                    &MATRIX_TEMPLATES
                }
            }

            impl Version for $name {
                const VERSION: usize = $number;
            }
        )+
    };
}

define_versions!(
    Version1 => 1,
    Version2 => 2,
    Version3 => 3,
    Version4 => 4,
    Version5 => 5,
    Version6 => 6,
    Version7 => 7,
    Version8 => 8,
    Version9 => 9,
    Version10 => 10,
    Version11 => 11,
    Version12 => 12,
    Version13 => 13,
    Version14 => 14,
    Version15 => 15,
    Version16 => 16,
    Version17 => 17,
    Version18 => 18,
    Version19 => 19,
    Version20 => 20,
    Version21 => 21,
    Version22 => 22,
    Version23 => 23,
    Version24 => 24,
    Version25 => 25,
    Version26 => 26,
    Version27 => 27,
    Version28 => 28,
    Version29 => 29,
    Version30 => 30,
    Version31 => 31,
    Version32 => 32,
    Version33 => 33,
    Version34 => 34,
    Version35 => 35,
    Version36 => 36,
    Version37 => 37,
    Version38 => 38,
    Version39 => 39,
    Version40 => 40,
);
