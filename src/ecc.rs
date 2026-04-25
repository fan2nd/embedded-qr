use crate::{
    helper::{block_count, ecc_codewords},
    types::EccLevel,
    version::Version,
};

pub(crate) const MAX_ECC_CODEWORDS: usize = 30;
const ECC_LEVELS: [EccLevel; 4] = [EccLevel::L, EccLevel::M, EccLevel::Q, EccLevel::H];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionGeneratorTable {
    level_indices: [u8; 4],
    degrees: [u8; 4],
    coefficients: [[u8; MAX_ECC_CODEWORDS]; 4],
}

impl VersionGeneratorTable {
    pub(crate) const fn build<T: Version>() -> Self {
        let mut level_indices = [0u8; 4];
        let mut degrees = [0u8; 4];
        let mut coefficients = [[0u8; MAX_ECC_CODEWORDS]; 4];
        let mut unique_count = 0usize;
        let mut level_index = 0usize;

        while level_index < ECC_LEVELS.len() {
            let level = ECC_LEVELS[level_index];
            let degree = (ecc_codewords::<T>(level) / block_count::<T>(level)) as u8;
            let existing = Self::find_degree(&degrees, unique_count, degree);
            let table_index = if existing < unique_count {
                existing
            } else {
                degrees[unique_count] = degree;
                coefficients[unique_count] =
                    ReedSolomon::build_generator_coefficients(degree as usize);
                unique_count += 1;
                unique_count - 1
            };
            level_indices[level_index] = table_index as u8;
            level_index += 1;
        }

        Self {
            level_indices,
            degrees,
            coefficients,
        }
    }

    const fn find_degree(degrees: &[u8; 4], count: usize, target: u8) -> usize {
        let mut index = 0usize;
        while index < count {
            if degrees[index] == target {
                return index;
            }
            index += 1;
        }
        count
    }

    pub(crate) fn for_level(&self, level: EccLevel) -> &[u8] {
        let index = self.level_indices[level.index()] as usize;
        let degree = self.degrees[index] as usize;
        &self.coefficients[index][..degree]
    }

    #[cfg(test)]
    fn unique_degree_count(&self) -> usize {
        let mut count = 0usize;
        while count < self.degrees.len() && self.degrees[count] != 0 {
            count += 1;
        }
        count
    }
}

pub(crate) struct ReedSolomon;

impl ReedSolomon {
    const EXP_TABLE: [u8; 255] = Self::build_exp_table();
    const LOG_TABLE: [u8; 256] = Self::build_log_table();

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn generate_ecc_into(data: &[u8], out: &mut [u8]) {
        let generator = Self::build_generator_coefficients(out.len());
        Self::generate_ecc_with_generator(data, out, &generator[..out.len()]);
    }

    pub(crate) fn generate_ecc_with_generator(data: &[u8], out: &mut [u8], generator: &[u8]) {
        debug_assert_eq!(out.len(), generator.len());
        out.fill(0);
        if out.is_empty() {
            return;
        }

        for &byte in data {
            let factor = byte ^ out[0];
            let mut index = 0usize;
            while index + 1 < out.len() {
                out[index] = out[index + 1] ^ Self::gf_mul(generator[index], factor);
                index += 1;
            }
            out[out.len() - 1] = Self::gf_mul(generator[out.len() - 1], factor);
        }
    }

    pub(crate) const fn build_generator_coefficients(degree: usize) -> [u8; MAX_ECC_CODEWORDS] {
        let mut result = [0u8; MAX_ECC_CODEWORDS];
        result[degree - 1] = 1;
        let mut root = 1u8;
        let mut outer = 0usize;

        while outer < degree {
            let mut index = 0usize;
            while index < degree {
                result[index] = Self::gf_mul(result[index], root);
                if index + 1 < degree {
                    result[index] ^= result[index + 1];
                }
                index += 1;
            }
            root = Self::gf_mul(root, 0x02);
            outer += 1;
        }

        result
    }

    const fn gf_mul(left: u8, right: u8) -> u8 {
        if left == 0 || right == 0 {
            return 0;
        }

        let log_left = Self::LOG_TABLE[left as usize] as usize;
        let log_right = Self::LOG_TABLE[right as usize] as usize;
        Self::EXP_TABLE[(log_left + log_right) % 255]
    }

    const fn xtime(value: u8) -> u8 {
        let shifted = value << 1;
        if (value & 0x80) != 0 {
            shifted ^ 0x1D
        } else {
            shifted
        }
    }

    const fn build_exp_table() -> [u8; 255] {
        let mut table = [0u8; 255];
        let mut value = 1u8;
        let mut index = 0usize;

        while index < 255 {
            table[index] = value;
            value = Self::xtime(value);
            index += 1;
        }

        table
    }

    const fn build_log_table() -> [u8; 256] {
        let exp = Self::build_exp_table();
        let mut table = [0u8; 256];
        let mut index = 0usize;

        while index < 255 {
            table[exp[index] as usize] = index as u8;
            index += 1;
        }

        table
    }
}

#[cfg(test)]
mod tests {
    use super::{ReedSolomon, VersionGeneratorTable};
    use crate::{
        encoder::EncodedData,
        helper::ecc_codewords,
        types::{EccLevel, EncodeMode},
        version::{Version1, Version17, new_data_ecc_buf},
    };

    #[test]
    fn zero_data_produces_zero_ecc() {
        let mut ecc = [0u8; 7];
        ReedSolomon::generate_ecc_into(&[0; 19], &mut ecc);
        assert_eq!(&ecc, &[0; 7]);
    }

    #[test]
    fn lookup_tables_encode_generator_powers() {
        assert_eq!(ReedSolomon::EXP_TABLE[0], 1);
        assert_eq!(ReedSolomon::EXP_TABLE[1], 2);
        assert_eq!(ReedSolomon::EXP_TABLE[8], 29);
        assert_eq!(ReedSolomon::LOG_TABLE[1], 0);
        assert_eq!(ReedSolomon::LOG_TABLE[2], 1);
        assert_eq!(ReedSolomon::LOG_TABLE[29], 8);
        assert_eq!(
            &ReedSolomon::build_generator_coefficients(7)[..7],
            &[127, 122, 154, 164, 11, 68, 117]
        );
    }

    #[test]
    fn version_bound_generator_tables_store_only_used_degrees() {
        let version1 = VersionGeneratorTable::build::<Version1>();
        assert_eq!(version1.unique_degree_count(), 4);
        assert_eq!(
            version1.for_level(EccLevel::L),
            &[127, 122, 154, 164, 11, 68, 117]
        );

        let version17 = VersionGeneratorTable::build::<Version17>();
        assert_eq!(version17.unique_degree_count(), 1);
        assert_eq!(
            version17.for_level(EccLevel::L),
            version17.for_level(EccLevel::H)
        );
    }

    #[test]
    fn append_ecc_writes_expected_length() {
        let mut codewords = new_data_ecc_buf::<Version1>();
        let encoded = EncodedData::<Version1>::encode(
            b"HELLO WORLD",
            EncodeMode::Alphanumeric,
            EccLevel::L,
            &mut codewords,
        )
        .unwrap();
        let ecc_len = ecc_codewords::<Version1>(EccLevel::L);
        let (data, remainder) = codewords.as_mut().split_at_mut(encoded.data_len);
        ReedSolomon::generate_ecc_into(&data[..encoded.data_len], &mut remainder[..ecc_len]);
        assert_eq!(codewords.as_ref().len(), 26);
        assert_ne!(&codewords.as_ref()[19..26], &[0; 7]);
    }

    #[test]
    fn ecc_matches_reference_for_all_version1_levels() {
        let cases = [
            (EccLevel::L, &[110, 57, 221, 152, 142, 219, 31][..]),
            (
                EccLevel::M,
                &[77, 221, 133, 19, 219, 95, 71, 115, 154, 101][..],
            ),
            (
                EccLevel::Q,
                &[197, 126, 205, 39, 143, 18, 53, 138, 62, 172, 29, 19, 74][..],
            ),
            (
                EccLevel::H,
                &[
                    109, 149, 156, 18, 217, 41, 246, 36, 42, 84, 46, 225, 190, 218, 251, 27, 196,
                ][..],
            ),
        ];

        for (level, expected) in cases {
            let mut codewords = new_data_ecc_buf::<Version1>();
            let encoded = EncodedData::<Version1>::encode(
                b"HELLO",
                EncodeMode::Alphanumeric,
                level,
                &mut codewords,
            )
            .unwrap();
            let ecc_len = ecc_codewords::<Version1>(level);
            let (data, remainder) = codewords.as_mut().split_at_mut(encoded.data_len);
            ReedSolomon::generate_ecc_into(&data[..encoded.data_len], &mut remainder[..ecc_len]);
            assert_eq!(&codewords.as_ref()[encoded.data_len..], expected);
        }
    }
}
