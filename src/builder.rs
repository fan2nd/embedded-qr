use core::marker::PhantomData;

use crate::{
    ecc::ReedSolomon,
    encoder::EncodedData,
    helper::best_ecc_level,
    interleave::interleave,
    matrix::QrMatrix,
    types::{EccLevel, EncodeMode, QrError},
    version::{Version, new_data_ecc_buf},
};

#[derive(Debug, Clone, Copy)]
/// Builds a QR code matrix for a specific typed QR version.
///
/// `QrBuilder<T>` keeps the version at the type level, which lets the crate
/// stay allocation-free and size its internal buffers at compile time.
pub struct QrBuilder<T: Version> {
    mode: Option<EncodeMode>,
    ecc_level: Option<EccLevel>,
    _marker: PhantomData<T>,
}

impl<T: Version> QrBuilder<T> {
    /// Creates a builder with automatic mode detection and ECC selection enabled.
    ///
    /// When `with_mode` is not used, `build` detects the most compact
    /// supported encoding mode automatically. You can override that choice
    /// explicitly with `with_mode`. When `with_ecc_level` is not used, it
    /// picks the strongest error-correction level that still fits the payload.
    /// You can override that choice explicitly with `with_ecc_level`.
    pub fn new() -> Self {
        Self {
            mode: None,
            ecc_level: None,
            _marker: PhantomData,
        }
    }

    /// Forces a specific encoding mode for the payload.
    ///
    /// If the data does not fit the selected mode, `build` returns
    /// [`QrError::DataInvalid`].
    pub fn with_mode(mut self, mode: EncodeMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Forces a specific error-correction level for the final symbol.
    pub fn with_ecc_level(mut self, ecc_level: EccLevel) -> Self {
        self.ecc_level = Some(ecc_level);
        self
    }

    /// Encodes `data` and returns the final QR matrix.
    ///
    /// The builder automatically chooses the most compact supported encoding
    /// mode unless `with_mode` was used. It always chooses the best mask. If
    /// ECC was not pinned with `with_ecc_level`, it also chooses the strongest
    /// fitting ECC level.
    ///
    /// Returns [`QrError::Overflow`] when the payload does not fit in the
    /// selected version.
    pub fn build(self, data: &[u8]) -> Result<QrMatrix<T>, QrError> {
        let mut codewords = new_data_ecc_buf::<T>();
        let mode = self.mode.unwrap_or_else(|| EncodeMode::choose(data));
        let ecc_level = match self.ecc_level {
            Some(ecc_level) => ecc_level,
            None => best_ecc_level::<T>(mode, data.len())?,
        };
        let encoded = EncodedData::<T>::encode(data, mode, ecc_level, &mut codewords)?;
        let generators = T::RS_GENERATORS;
        let generator = generators.for_level(ecc_level);
        interleave::<T, _>(&mut codewords, encoded, ecc_level, |block_data, ecc_out| {
            ReedSolomon::generate_ecc_with_generator(block_data, ecc_out, generator)
        });

        Ok(QrMatrix::<T>::from_codewords(codewords.as_ref(), ecc_level))
    }
}

impl<T: Version> Default for QrBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::QrBuilder;
    use crate::{
        encoder::EncodedData,
        helper::best_ecc_level,
        types::{EccLevel, EncodeMode, QrError},
        version::{Version1, new_data_ecc_buf},
    };

    #[test]
    fn auto_ecc_prefers_highest_level_that_fits() {
        assert_eq!(
            best_ecc_level::<Version1>(EncodeMode::Alphanumeric, 5),
            Ok(EccLevel::H)
        );
        assert_eq!(
            best_ecc_level::<Version1>(EncodeMode::Bytes, 8),
            Ok(EccLevel::Q)
        );
        assert_eq!(
            best_ecc_level::<Version1>(EncodeMode::Bytes, 12),
            Ok(EccLevel::M)
        );
        assert_eq!(
            best_ecc_level::<Version1>(EncodeMode::Bytes, 17),
            Ok(EccLevel::L)
        );
    }

    #[test]
    fn auto_ecc_reports_overflow_when_nothing_fits() {
        assert_eq!(
            best_ecc_level::<Version1>(EncodeMode::Bytes, 18),
            Err(QrError::Overflow)
        );
    }

    #[test]
    fn builder_keeps_manual_overrides() {
        let matrix = QrBuilder::<Version1>::new()
            .with_mode(EncodeMode::Alphanumeric)
            .with_ecc_level(EccLevel::L)
            .build(b"HELLO WORLD")
            .unwrap();
        assert_eq!(matrix.width(), 21);
    }

    #[test]
    fn builder_rejects_invalid_manual_mode() {
        let result = QrBuilder::<Version1>::new()
            .with_mode(EncodeMode::Numeric)
            .build(b"HELLO");
        assert!(matches!(result, Err(QrError::DataInvalid)));
    }

    #[test]
    fn best_encoding_returns_encoded_payload_once() {
        let mut codewords = new_data_ecc_buf::<Version1>();
        let ecc_level = best_ecc_level::<Version1>(EncodeMode::Bytes, 12).unwrap();
        let encoded = EncodedData::<Version1>::encode(
            b"abcdefgh1234",
            EncodeMode::Bytes,
            ecc_level,
            &mut codewords,
        )
        .unwrap();
        assert_eq!(ecc_level, EccLevel::M);
        assert_eq!(encoded.data_len, 16);
    }
}
