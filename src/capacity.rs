use crate::{
    helper,
    types::{EccLevel, EncodeMode},
    version::Version,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Capacity metadata for a fixed QR version and ECC level.
pub struct VersionCapacity {
    /// Total codewords in the symbol, including data and ECC.
    pub total_codewords: usize,
    /// Data codewords available after ECC overhead.
    pub data_codewords: usize,
    /// ECC codewords appended to the payload.
    pub ecc_codewords: usize,
    /// Number of Reed-Solomon blocks for this version/ECC combination.
    pub block_count: usize,
}

/// Returns capacity metadata for a fixed QR version and ECC level.
pub const fn info<T: Version>(level: EccLevel) -> VersionCapacity {
    VersionCapacity {
        total_codewords: helper::total_codewords::<T>(),
        data_codewords: helper::data_capacity::<T>(level),
        ecc_codewords: helper::ecc_codewords::<T>(level),
        block_count: helper::block_count::<T>(level),
    }
}

/// Returns the number of bits needed to encode a payload length in a given mode.
///
/// The result includes the 4-bit mode indicator and the version-dependent
/// character-count field, but does not include terminator or pad bytes.
pub const fn encoded_bits<T: Version>(mode: EncodeMode, len: usize) -> Option<usize> {
    helper::encoded_bits::<T>(mode, len)
}

/// Returns whether `len` input units fit in the selected version/mode/ECC tuple.
///
/// For `Numeric` and `Alphanumeric`, `len` is the number of characters.
/// For `Bytes`, `len` is the number of bytes.
pub const fn fits<T: Version>(mode: EncodeMode, len: usize, level: EccLevel) -> bool {
    helper::fits::<T>(mode, len, level)
}

/// Returns the largest payload length that fits the selected version/mode/ECC tuple.
///
/// For `Numeric` and `Alphanumeric`, the returned value is in characters.
/// For `Bytes`, it is in bytes.
pub const fn max_payload_len<T: Version>(mode: EncodeMode, level: EccLevel) -> usize {
    let header_bits = 4 + mode.counter_bits::<T>();
    let available_bits = helper::data_capacity::<T>(level).saturating_mul(8);
    if available_bits < header_bits {
        return 0;
    }

    let payload_bits = available_bits - header_bits;
    match mode {
        EncodeMode::Numeric => {
            let groups = payload_bits / 10;
            let remainder = payload_bits % 10;
            groups * 3
                + if remainder >= 7 {
                    2
                } else if remainder >= 4 {
                    1
                } else {
                    0
                }
        }
        EncodeMode::Alphanumeric => {
            let pairs = payload_bits / 11;
            let remainder = payload_bits % 11;
            (pairs * 2) + if remainder >= 6 { 1 } else { 0 }
        }
        EncodeMode::Bytes => payload_bits / 8,
    }
}

#[cfg(test)]
mod tests {
    use super::{encoded_bits, fits, info, max_payload_len};
    use crate::{
        types::{EccLevel, EncodeMode},
        version::{
            Version, Version1, Version2, Version3, Version4, Version5, Version6, Version7,
            Version8, Version9, Version10, Version11, Version12, Version13, Version14, Version15,
            Version16, Version17, Version18, Version19, Version20, Version21, Version22, Version23,
            Version24, Version25, Version26, Version27, Version28, Version29, Version30, Version31,
            Version32, Version33, Version34, Version35, Version36, Version37, Version38, Version39,
            Version40,
        },
    };

    const ECC_LEVELS: [EccLevel; 4] = [EccLevel::L, EccLevel::M, EccLevel::Q, EccLevel::H];
    const MODES: [EncodeMode; 3] = [
        EncodeMode::Numeric,
        EncodeMode::Alphanumeric,
        EncodeMode::Bytes,
    ];

    fn assert_capacity_boundaries<T: Version>() {
        for level in ECC_LEVELS {
            let capacity_bits = info::<T>(level).data_codewords * 8;
            for mode in MODES {
                let max = max_payload_len::<T>(mode, level);
                let bits_at_max =
                    encoded_bits::<T>(mode, max).expect("max payload length should encode");
                assert!(
                    bits_at_max <= capacity_bits,
                    "version {} {:?} {:?} max len {} exceeds capacity",
                    T::VERSION,
                    level,
                    mode,
                    max
                );
                assert!(
                    fits::<T>(mode, max, level),
                    "version {} {:?} {:?} max len {} should fit",
                    T::VERSION,
                    level,
                    mode,
                    max
                );

                let next = max + 1;
                let next_fits = fits::<T>(mode, next, level);
                let bits_at_next = encoded_bits::<T>(mode, next);
                assert!(
                    !next_fits,
                    "version {} {:?} {:?} next len {} should overflow",
                    T::VERSION,
                    level,
                    mode,
                    next
                );
                if let Some(bits) = bits_at_next {
                    assert!(
                        bits > capacity_bits,
                        "version {} {:?} {:?} next len {} should require more than {} bits",
                        T::VERSION,
                        level,
                        mode,
                        next,
                        capacity_bits
                    );
                }
            }
        }
    }

    #[test]
    fn version_capacity_matches_known_version1_tables() {
        let capacity = info::<Version1>(EccLevel::M);
        assert_eq!(capacity.total_codewords, 26);
        assert_eq!(capacity.data_codewords, 16);
        assert_eq!(capacity.ecc_codewords, 10);
        assert_eq!(capacity.block_count, 1);
    }

    #[test]
    fn encoded_bits_matches_known_examples() {
        assert_eq!(encoded_bits::<Version1>(EncodeMode::Bytes, 12), Some(108));
        assert_eq!(encoded_bits::<Version1>(EncodeMode::Numeric, 41), Some(151));
    }

    #[test]
    fn fits_matches_known_version1_byte_limits() {
        assert!(fits::<Version1>(EncodeMode::Bytes, 17, EccLevel::L));
        assert!(!fits::<Version1>(EncodeMode::Bytes, 18, EccLevel::L));
        assert!(fits::<Version1>(EncodeMode::Bytes, 11, EccLevel::Q));
        assert!(!fits::<Version1>(EncodeMode::Bytes, 12, EccLevel::Q));
    }

    #[test]
    fn max_payload_len_matches_known_version1_limits() {
        assert_eq!(
            max_payload_len::<Version1>(EncodeMode::Numeric, EccLevel::L),
            41
        );
        assert_eq!(
            max_payload_len::<Version1>(EncodeMode::Alphanumeric, EccLevel::L),
            25
        );
        assert_eq!(
            max_payload_len::<Version1>(EncodeMode::Bytes, EccLevel::L),
            17
        );
    }

    #[test]
    fn capacity_boundaries_hold_for_all_versions() {
        assert_capacity_boundaries::<Version1>();
        assert_capacity_boundaries::<Version2>();
        assert_capacity_boundaries::<Version3>();
        assert_capacity_boundaries::<Version4>();
        assert_capacity_boundaries::<Version5>();
        assert_capacity_boundaries::<Version6>();
        assert_capacity_boundaries::<Version7>();
        assert_capacity_boundaries::<Version8>();
        assert_capacity_boundaries::<Version9>();
        assert_capacity_boundaries::<Version10>();
        assert_capacity_boundaries::<Version11>();
        assert_capacity_boundaries::<Version12>();
        assert_capacity_boundaries::<Version13>();
        assert_capacity_boundaries::<Version14>();
        assert_capacity_boundaries::<Version15>();
        assert_capacity_boundaries::<Version16>();
        assert_capacity_boundaries::<Version17>();
        assert_capacity_boundaries::<Version18>();
        assert_capacity_boundaries::<Version19>();
        assert_capacity_boundaries::<Version20>();
        assert_capacity_boundaries::<Version21>();
        assert_capacity_boundaries::<Version22>();
        assert_capacity_boundaries::<Version23>();
        assert_capacity_boundaries::<Version24>();
        assert_capacity_boundaries::<Version25>();
        assert_capacity_boundaries::<Version26>();
        assert_capacity_boundaries::<Version27>();
        assert_capacity_boundaries::<Version28>();
        assert_capacity_boundaries::<Version29>();
        assert_capacity_boundaries::<Version30>();
        assert_capacity_boundaries::<Version31>();
        assert_capacity_boundaries::<Version32>();
        assert_capacity_boundaries::<Version33>();
        assert_capacity_boundaries::<Version34>();
        assert_capacity_boundaries::<Version35>();
        assert_capacity_boundaries::<Version36>();
        assert_capacity_boundaries::<Version37>();
        assert_capacity_boundaries::<Version38>();
        assert_capacity_boundaries::<Version39>();
        assert_capacity_boundaries::<Version40>();
    }
}
