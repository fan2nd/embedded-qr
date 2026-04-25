use crate::{
    types::{EccLevel, Mask},
    version::{MatrixBuf, Version, matrix_templates},
};

#[derive(Debug, Clone)]
/// A fully encoded QR code matrix.
///
/// The matrix stores only the QR modules themselves. Quiet-zone handling and
/// any visual rendering are expected to be done by the caller.
pub struct QrMatrix<T: Version> {
    buffer: MatrixBuf<T>,
    ecc_level: EccLevel,
    mask: Mask,
}

impl<T: Version> QrMatrix<T> {
    pub(crate) fn from_codewords(codewords: &[u8], level: EccLevel) -> Self {
        let templates = matrix_templates::<T>();
        let mut buffer = templates.modules.clone();
        let reserved = templates.reserved.as_ref();
        Self::place_data(&mut buffer, reserved, codewords);
        let mut best_score = usize::MAX;
        let mut best_mask = Mask::M0;

        for mask in Mask::ALL {
            Self::apply_mask(&mut buffer, reserved, mask);
            Self::draw_format_bits(&mut buffer, level, mask);
            let score = Self::penalty_score(buffer.as_ref());
            if score < best_score {
                best_score = score;
                best_mask = mask;
            }
            Self::apply_mask(&mut buffer, reserved, mask);
        }

        Self::apply_mask(&mut buffer, reserved, best_mask);
        Self::draw_format_bits(&mut buffer, level, best_mask);

        Self {
            buffer,
            ecc_level: level,
            mask: best_mask,
        }
    }

    /// Returns the side length of the QR matrix in modules.
    pub fn width(&self) -> usize {
        T::WIDTH
    }

    /// Returns whether the module at `(x, y)` is dark.
    ///
    /// Coordinates are zero-based and must stay within the matrix bounds.
    pub fn get(&self, x: usize, y: usize) -> bool {
        assert!(x < T::WIDTH);
        assert!(y < T::WIDTH);
        Self::get_module(self.buffer.as_ref(), x, y)
    }

    /// Returns the ECC level used for this matrix.
    pub fn ecc_level(&self) -> EccLevel {
        self.ecc_level
    }

    /// Returns the mask pattern chosen for this matrix.
    pub fn mask(&self) -> Mask {
        self.mask
    }

    /// Iterates over all modules in row-major order.
    pub fn iter(&self) -> QrMatrixIter<'_, T> {
        QrMatrixIter {
            matrix: self,
            x: 0,
            y: 0,
        }
    }
    fn place_data(modules: &mut MatrixBuf<T>, reserved: &[u8], codewords: &[u8]) {
        let total_bits = codewords.len() * 8;
        let mut bit_index = 0usize;
        let mut upward = true;
        let mut right = T::WIDTH - 1;

        while right > 0 {
            if right == 6 {
                right -= 1;
            }

            for offset in 0..T::WIDTH {
                let y = if upward {
                    T::WIDTH - 1 - offset
                } else {
                    offset
                };
                for dx in 0..2 {
                    let x = right - dx;
                    if Self::is_reserved(reserved, x, y) {
                        continue;
                    }
                    let is_black =
                        bit_index < total_bits && Self::get_codeword_bit(codewords, bit_index);
                    Self::set_module(modules, x, y, is_black);
                    bit_index += 1;
                }
            }

            upward = !upward;
            if right < 2 {
                break;
            }
            right -= 2;
        }
    }

    fn apply_mask(modules: &mut MatrixBuf<T>, reserved: &[u8], mask: Mask) {
        for y in 0..T::WIDTH {
            for x in 0..T::WIDTH {
                if !Self::is_reserved(reserved, x, y) && mask.applies(x, y) {
                    let value = !Self::get_module(modules.as_ref(), x, y);
                    Self::set_module(modules, x, y, value);
                }
            }
        }
    }

    fn draw_format_bits(modules: &mut MatrixBuf<T>, level: EccLevel, mask: Mask) {
        let bits = mask.format_bits(level);

        for index in 0..=5 {
            Self::set_module(modules, 8, index, Self::get_format_bit(bits, index));
        }
        Self::set_module(modules, 8, 7, Self::get_format_bit(bits, 6));
        Self::set_module(modules, 8, 8, Self::get_format_bit(bits, 7));
        Self::set_module(modules, 7, 8, Self::get_format_bit(bits, 8));
        for index in 9..=14 {
            Self::set_module(modules, 14 - index, 8, Self::get_format_bit(bits, index));
        }

        for index in 0..=7 {
            Self::set_module(
                modules,
                T::WIDTH - 1 - index,
                8,
                Self::get_format_bit(bits, index),
            );
        }
        for index in 8..=14 {
            Self::set_module(
                modules,
                8,
                T::WIDTH - 15 + index,
                Self::get_format_bit(bits, index),
            );
        }
        Self::set_module(modules, 8, T::WIDTH - 8, true);
    }

    fn penalty_score(modules: &[u8]) -> usize {
        Self::penalty_rule1_rows(modules)
            + Self::penalty_rule1_cols(modules)
            + Self::penalty_rule2(modules)
            + Self::penalty_rule3_rows(modules)
            + Self::penalty_rule3_cols(modules)
            + Self::penalty_rule4(modules)
    }

    fn penalty_rule1_rows(modules: &[u8]) -> usize {
        let mut penalty = 0;
        for y in 0..T::WIDTH {
            penalty +=
                Self::penalty_for_line((0..T::WIDTH).map(|x| Self::get_module(modules, x, y)));
        }
        penalty
    }

    fn penalty_rule1_cols(modules: &[u8]) -> usize {
        let mut penalty = 0;
        for x in 0..T::WIDTH {
            penalty +=
                Self::penalty_for_line((0..T::WIDTH).map(|y| Self::get_module(modules, x, y)));
        }
        penalty
    }

    fn penalty_for_line<I>(mut iter: I) -> usize
    where
        I: Iterator<Item = bool>,
    {
        let Some(mut current) = iter.next() else {
            return 0;
        };
        let mut run_len = 1usize;
        let mut penalty = 0usize;

        for bit in iter {
            if bit == current {
                run_len += 1;
            } else {
                penalty += Self::penalty_for_run(run_len);
                current = bit;
                run_len = 1;
            }
        }

        penalty + Self::penalty_for_run(run_len)
    }

    fn penalty_rule2(modules: &[u8]) -> usize {
        let mut penalty = 0usize;
        for y in 0..(T::WIDTH - 1) {
            for x in 0..(T::WIDTH - 1) {
                let color = Self::get_module(modules, x, y);
                if Self::get_module(modules, x + 1, y) == color
                    && Self::get_module(modules, x, y + 1) == color
                    && Self::get_module(modules, x + 1, y + 1) == color
                {
                    penalty += 3;
                }
            }
        }
        penalty
    }

    fn penalty_rule3_rows(modules: &[u8]) -> usize {
        let mut penalty = 0usize;
        for y in 0..T::WIDTH {
            for x in 0..=(T::WIDTH - 11) {
                let mut bits = [false; 11];
                for index in 0..11 {
                    bits[index] = Self::get_module(modules, x + index, y);
                }
                if Self::is_finder_like_pattern(bits) {
                    penalty += 40;
                }
            }
        }
        penalty
    }

    fn penalty_rule3_cols(modules: &[u8]) -> usize {
        let mut penalty = 0usize;
        for x in 0..T::WIDTH {
            for y in 0..=(T::WIDTH - 11) {
                let mut bits = [false; 11];
                for index in 0..11 {
                    bits[index] = Self::get_module(modules, x, y + index);
                }
                if Self::is_finder_like_pattern(bits) {
                    penalty += 40;
                }
            }
        }
        penalty
    }

    fn penalty_rule4(modules: &[u8]) -> usize {
        let mut dark_modules = 0usize;
        for y in 0..T::WIDTH {
            for x in 0..T::WIDTH {
                if Self::get_module(modules, x, y) {
                    dark_modules += 1;
                }
            }
        }

        let total_modules = T::WIDTH * T::WIDTH;
        let deviation =
            dark_modules.saturating_mul(20).abs_diff(total_modules * 10) / total_modules;
        deviation * 10
    }

    fn set_module(modules: &mut MatrixBuf<T>, x: usize, y: usize, is_black: bool) {
        Self::set_bit(modules.as_mut(), y * T::WIDTH + x, is_black);
    }

    fn get_module(modules: &[u8], x: usize, y: usize) -> bool {
        Self::get_bit(modules, y * T::WIDTH + x)
    }

    fn is_reserved(reserved: &[u8], x: usize, y: usize) -> bool {
        Self::get_bit(reserved, y * T::WIDTH + x)
    }

    fn set_bit(buffer: &mut [u8], index: usize, value: bool) {
        let byte_index = index >> 3;
        let offset = index & 0x07;
        let mask = 1u8 << offset;
        if value {
            buffer[byte_index] |= mask;
        } else {
            buffer[byte_index] &= !mask;
        }
    }

    fn get_bit(buffer: &[u8], index: usize) -> bool {
        let byte_index = index >> 3;
        let offset = index & 0x07;
        ((buffer[byte_index] >> offset) & 1) != 0
    }

    fn get_codeword_bit(codewords: &[u8], bit_index: usize) -> bool {
        let byte = codewords[bit_index >> 3];
        let offset = 7 - (bit_index & 0x07);
        ((byte >> offset) & 1) != 0
    }

    fn penalty_for_run(run_len: usize) -> usize {
        if run_len >= 5 { run_len - 2 } else { 0 }
    }

    fn is_finder_like_pattern(bits: [bool; 11]) -> bool {
        const PATTERN_A: [bool; 11] = [
            true, false, true, true, true, false, true, false, false, false, false,
        ];
        const PATTERN_B: [bool; 11] = [
            false, false, false, false, true, false, true, true, true, false, true,
        ];
        bits == PATTERN_A || bits == PATTERN_B
    }

    fn get_format_bit(bits: u16, index: usize) -> bool {
        ((bits >> index) & 1) != 0
    }

    #[cfg(test)]
    fn alignment_pattern_positions() -> ([usize; 7], usize) {
        if T::VERSION == 1 {
            return ([0; 7], 0);
        }

        let count = (T::VERSION / 7) + 2;
        let step = if T::VERSION == 32 {
            26
        } else {
            (((T::VERSION * 4) + (count * 2) + 1) / ((count * 2) - 2)) * 2
        };
        let mut positions = [0usize; 7];
        positions[0] = 6;
        let mut pos = T::WIDTH - 7;
        let mut index = count - 1;

        while index > 0 {
            positions[index] = pos;
            if index == 1 {
                break;
            }
            pos -= step;
            index -= 1;
        }

        (positions, count)
    }
}

/// Iterator over a [`QrMatrix`] in row-major order.
///
/// Each item is `(x, y, is_dark)`.
pub struct QrMatrixIter<'d, T: Version> {
    matrix: &'d QrMatrix<T>,
    x: usize,
    y: usize,
}

impl<'d, T: Version> Iterator for QrMatrixIter<'d, T> {
    type Item = (usize, usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= T::WIDTH {
            return None;
        }

        let x = self.x;
        let y = self.y;
        let dark = QrMatrix::<T>::get_module(self.matrix.buffer.as_ref(), x, y);
        self.x += 1;
        if self.x == T::WIDTH {
            self.x = 0;
            self.y += 1;
        }
        Some((x, y, dark))
    }
}

#[cfg(test)]
mod tests {
    use super::QrMatrix;
    use crate::{
        builder::QrBuilder,
        types::{EccLevel, Mask},
        version::{Version1, Version2, Version7, Version14, Version32, Version36, Version40},
    };

    fn matrix_to_string(matrix: &crate::QrMatrix<Version1>) -> std::string::String {
        let mut output = std::string::String::new();
        for y in 0..matrix.width() {
            for x in 0..matrix.width() {
                output.push(if matrix.get(x, y) { '#' } else { '.' });
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn format_bits_match_known_reference() {
        assert_eq!(Mask::M0.format_bits(EccLevel::M), 0x5412);
        assert_eq!(Mask::M7.format_bits(EccLevel::H), 0x083B);
        assert!(!QrMatrix::<Version1>::get_format_bit(
            Mask::M0.format_bits(EccLevel::M),
            0
        ));
        assert!(QrMatrix::<Version1>::get_format_bit(
            Mask::M0.format_bits(EccLevel::M),
            1
        ));
    }

    #[test]
    fn matrix_iter_covers_every_module() {
        let matrix = QrBuilder::<Version1>::new().build(b"HELLO WORLD").unwrap();
        assert_eq!(matrix.iter().count(), 21 * 21);
        assert_eq!(matrix.iter().next(), Some((0, 0, true)));
    }

    #[test]
    fn matrix_width_matches_version() {
        let matrix = QrBuilder::<Version1>::new()
            .with_ecc_level(EccLevel::L)
            .build(b"HELLO WORLD")
            .unwrap();
        assert_eq!(matrix.width(), 21);
        assert_eq!(matrix.ecc_level(), EccLevel::L);
    }

    #[test]
    fn finder_patterns_land_in_expected_positions() {
        let matrix = QrBuilder::<Version1>::new().build(b"HELLO WORLD").unwrap();
        assert!(matrix.get(0, 0));
        assert!(matrix.get(6, 0));
        assert!(matrix.get(0, 6));
        assert!(matrix.get(20, 0));
        assert!(matrix.get(0, 20));
        assert!(!matrix.get(7, 7));
    }

    #[test]
    fn auto_mask_is_reproducible() {
        let left = QrBuilder::<Version1>::new().build(b"01234567").unwrap();
        let right = QrBuilder::<Version1>::new().build(b"01234567").unwrap();
        assert_eq!(left.mask(), right.mask());
        for (x, y, dark) in left.iter() {
            assert_eq!(dark, right.get(x, y));
        }
    }

    #[test]
    fn auto_mask_builds_matrix() {
        let matrix = QrBuilder::<Version1>::new().build(b"HELLO WORLD").unwrap();
        assert!(matrix.iter().any(|(_, _, dark)| dark));
        assert_eq!(matrix.ecc_level(), EccLevel::Q);
        assert_eq!(matrix.mask(), Mask::M6);
    }

    #[test]
    fn auto_selection_is_exposed_on_result() {
        let short = QrBuilder::<Version1>::new().build(b"HELLO").unwrap();
        let longer = QrBuilder::<Version1>::new().build(b"abcdefgh1234").unwrap();
        assert_eq!(short.ecc_level(), EccLevel::H);
        assert_eq!(short.mask(), Mask::M3);
        assert_eq!(longer.ecc_level(), EccLevel::M);
    }

    #[test]
    fn alignment_pattern_positions_match_known_versions() {
        assert_eq!(
            QrMatrix::<Version2>::alignment_pattern_positions(),
            ([6, 18, 0, 0, 0, 0, 0], 2)
        );
        assert_eq!(
            QrMatrix::<Version7>::alignment_pattern_positions(),
            ([6, 22, 38, 0, 0, 0, 0], 3)
        );
        assert_eq!(
            QrMatrix::<Version14>::alignment_pattern_positions(),
            ([6, 26, 46, 66, 0, 0, 0], 4)
        );
        assert_eq!(
            QrMatrix::<Version32>::alignment_pattern_positions(),
            ([6, 34, 60, 86, 112, 138, 0], 6)
        );
        assert_eq!(
            QrMatrix::<Version36>::alignment_pattern_positions(),
            ([6, 24, 50, 76, 102, 128, 154], 7)
        );
        assert_eq!(
            QrMatrix::<Version40>::alignment_pattern_positions(),
            ([6, 30, 58, 86, 114, 142, 170], 7)
        );
    }

    #[test]
    fn version2_draws_alignment_pattern() {
        let matrix = QrBuilder::<Version2>::new()
            .with_ecc_level(EccLevel::L)
            .build(b"HELLO WORLD")
            .unwrap();

        assert_eq!(matrix.width(), 25);
        assert!(matrix.get(18, 18));
        assert!(matrix.get(16, 18));
        assert!(matrix.get(20, 18));
        assert!(matrix.get(18, 16));
        assert!(matrix.get(18, 20));
        assert!(!matrix.get(17, 18));
        assert!(!matrix.get(18, 17));
        assert!(!matrix.get(17, 17));
    }

    #[test]
    fn version7_draws_known_version_info_bits() {
        const VERSION7_BITS: u32 = 0x07C94;

        let matrix = QrBuilder::<Version7>::new()
            .with_ecc_level(EccLevel::L)
            .build(b"HELLO WORLD")
            .unwrap();

        assert_eq!(matrix.width(), 45);
        for index in 0..18 {
            let expected = ((VERSION7_BITS >> index) & 1) != 0;
            let x = 45 - 11 + (index % 3);
            let y = index / 3;
            assert_eq!(matrix.get(x, y), expected);
            assert_eq!(matrix.get(y, x), expected);
        }
    }

    #[test]
    fn hello_world_matrix_matches_reference_snapshot() {
        let matrix = QrBuilder::<Version1>::new()
            .with_ecc_level(EccLevel::L)
            .build(b"HELLO WORLD")
            .unwrap();
        assert_eq!(
            matrix_to_string(&matrix),
            concat!(
                "#######...#.#.#######\n",
                "#.....#.#.#.#.#.....#\n",
                "#.###.#.#.##..#.###.#\n",
                "#.###.#.....#.#.###.#\n",
                "#.###.#.#####.#.###.#\n",
                "#.....#.###...#.....#\n",
                "#######.#.#.#.#######\n",
                "........#............\n",
                "##.#..##..###.###.##.\n",
                "###.##..#.##....#...#\n",
                "#.#...#..#.#.##..#.#.\n",
                "#.####.###..####..###\n",
                "...#####.###..###.#.#\n",
                "........#....##.#.###\n",
                "#######.#..##.##..#.#\n",
                "#.....#...#...##.#...\n",
                "#.###.#..##.####.##.#\n",
                "#.###.#.#.#..###.#.##\n",
                "#.###.#...##.###.#..#\n",
                "#.....#.#.###...##..#\n",
                "#######.#.#..#.#.#...\n",
            )
        );
    }

    #[test]
    fn numeric_matrix_matches_reference_snapshot() {
        let matrix = QrBuilder::<Version1>::new()
            .with_ecc_level(EccLevel::M)
            .build(b"01234567")
            .unwrap();
        assert_eq!(
            matrix_to_string(&matrix),
            concat!(
                "#######...###.#######\n",
                "#.....#.###...#.....#\n",
                "#.###.#..##...#.###.#\n",
                "#.###.#..#.##.#.###.#\n",
                "#.###.#.##.##.#.###.#\n",
                "#.....#....#..#.....#\n",
                "#######.#.#.#.#######\n",
                ".....................\n",
                "#.#.#.#...#.#...#..#.\n",
                "##.#....#.##.#.#...#.\n",
                "...##.###.##.###.###.\n",
                "##..##.#.#.###.##..#.\n",
                "..#..###.###.###....#\n",
                "........#.#...#....#.\n",
                "#######.....#...#...#\n",
                "#.....#...#...#..#.##\n",
                "#.###.#.###.#.#.###.#\n",
                "#.###.#..#.#.#.#.###.\n",
                "#.###.#.##.#.###..#.#\n",
                "#.....#....###.###...\n",
                "#######.#..#.###..#.#\n",
            )
        );
    }
}
