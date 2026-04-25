use crate::{
    types::{EccLevel, EncodeMode, QrError},
    version::Version,
};

pub const fn total_codewords<T: Version>() -> usize {
    const CODEWORDS: [usize; 40] = [
        26, 44, 70, 100, 134, 172, 196, 242, 292, 346, 404, 466, 532, 581, 655, 733, 815, 901, 991,
        1085, 1156, 1258, 1364, 1474, 1588, 1706, 1828, 1921, 2051, 2185, 2323, 2465, 2611, 2761,
        2876, 3034, 3196, 3362, 3532, 3706,
    ];

    CODEWORDS[T::VERSION - 1]
}

pub const fn data_capacity<T: Version>(level: EccLevel) -> usize {
    T::CAPACITY.for_level(level)
}

pub const fn ecc_codewords<T: Version>(level: EccLevel) -> usize {
    T::TOTAL_CODEWORDS - data_capacity::<T>(level)
}

pub const fn block_count<T: Version>(level: EccLevel) -> usize {
    T::BLOCKS.for_level(level)
}

pub const fn max_ecc_codewords_per_block<T: Version>() -> usize {
    let l = ecc_codewords::<T>(EccLevel::L) / block_count::<T>(EccLevel::L);
    let m = ecc_codewords::<T>(EccLevel::M) / block_count::<T>(EccLevel::M);
    let q = ecc_codewords::<T>(EccLevel::Q) / block_count::<T>(EccLevel::Q);
    let h = ecc_codewords::<T>(EccLevel::H) / block_count::<T>(EccLevel::H);
    let mut max = l;
    if m > max {
        max = m;
    }
    if q > max {
        max = q;
    }
    if h > max {
        max = h;
    }
    max
}

pub(crate) fn best_ecc_level<T: Version>(
    mode: EncodeMode,
    len: usize,
) -> Result<EccLevel, QrError> {
    [EccLevel::H, EccLevel::Q, EccLevel::M, EccLevel::L]
        .into_iter()
        .find(|&level| fits::<T>(mode, len, level))
        .ok_or(QrError::Overflow)
}

pub(crate) const fn fits<T: Version>(mode: EncodeMode, len: usize, level: EccLevel) -> bool {
    match encoded_bits::<T>(mode, len) {
        Some(bits) => bits <= data_capacity::<T>(level) * 8,
        None => false,
    }
}

pub(crate) const fn encoded_bits<T: Version>(mode: EncodeMode, len: usize) -> Option<usize> {
    let header_bits = 4 + mode.counter_bits::<T>();
    match payload_bits(mode, len) {
        Some(payload_bits) => header_bits.checked_add(payload_bits),
        None => None,
    }
}

const fn payload_bits(mode: EncodeMode, len: usize) -> Option<usize> {
    match mode {
        EncodeMode::Numeric => {
            let groups = len / 3;
            let remainder = len % 3;
            let remainder_bits = match remainder {
                0 => 0,
                1 => 4,
                _ => 7,
            };
            match groups.checked_mul(10) {
                Some(bits) => bits.checked_add(remainder_bits),
                None => None,
            }
        }
        EncodeMode::Alphanumeric => {
            let pairs = len / 2;
            let remainder = len % 2;
            let remainder_bits = if remainder == 1 { 6 } else { 0 };
            match pairs.checked_mul(11) {
                Some(bits) => bits.checked_add(remainder_bits),
                None => None,
            }
        }
        EncodeMode::Bytes => len.checked_mul(8),
    }
}

#[cfg(test)]
mod tests {
    use super::best_ecc_level;
    use crate::{
        types::{EccLevel, EncodeMode, QrError},
        version::Version1,
    };

    #[test]
    fn best_ecc_level_prefers_highest_fitting_level() {
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
    fn best_ecc_level_reports_overflow() {
        assert_eq!(
            best_ecc_level::<Version1>(EncodeMode::Bytes, 18),
            Err(QrError::Overflow)
        );
    }
}
