use crate::{
    encoder::EncodedData,
    helper::{block_count, data_capacity, ecc_codewords},
    types::EccLevel,
    version::{DataEccBuf, Version, new_interleave_ecc_buf, new_interleave_visited_buf},
};

pub(crate) fn interleave<T: Version, F>(
    codewords: &mut DataEccBuf<T>,
    encoded: EncodedData<T>,
    level: EccLevel,
    mut write_ecc: F,
) where
    F: FnMut(&[u8], &mut [u8]),
{
    let layout = BlockLayout::<T>::new(level);
    let mut ecc = new_interleave_ecc_buf::<T>();
    for block in 0..layout.block_count {
        let block_offset = layout.block_offset(block);
        let block_len = layout.data_codewords(block);
        let block_data = &codewords.as_ref()[block_offset..block_offset + block_len];
        let ecc_slice = &mut ecc.as_mut()[..layout.ecc_codewords_per_block];
        write_ecc(block_data, ecc_slice);
        let output = codewords.as_mut();
        for ecc_index in 0..layout.ecc_codewords_per_block {
            output[encoded.data_len + (ecc_index * layout.block_count) + block] =
                ecc_slice[ecc_index];
        }
    }

    layout.interleave_data_in_place(&mut codewords.as_mut()[..encoded.data_len]);
}

#[derive(Debug, Clone, Copy)]
struct BlockLayout<T: Version> {
    block_count: usize,
    short_block_count: usize,
    short_data_codewords: usize,
    max_data_codewords: usize,
    ecc_codewords_per_block: usize,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Version> BlockLayout<T> {
    fn new(level: EccLevel) -> Self {
        let total_data_codewords = data_capacity::<T>(level);
        let block_count = block_count::<T>(level);
        let long_block_count = total_data_codewords % block_count;
        let short_data_codewords = total_data_codewords / block_count;

        Self {
            block_count,
            short_block_count: block_count - long_block_count,
            short_data_codewords,
            max_data_codewords: short_data_codewords + usize::from(long_block_count > 0),
            ecc_codewords_per_block: ecc_codewords::<T>(level) / block_count,
            _marker: core::marker::PhantomData,
        }
    }

    fn data_codewords(&self, block_index: usize) -> usize {
        self.short_data_codewords + usize::from(block_index >= self.short_block_count)
    }

    fn block_offset(&self, block_index: usize) -> usize {
        (block_index * self.short_data_codewords)
            + block_index.saturating_sub(self.short_block_count)
    }

    fn source_to_interleaved_index(&self, source_index: usize) -> usize {
        let short_total = self.short_block_count * self.short_data_codewords;
        let (block, row) = if source_index < short_total {
            (
                source_index / self.short_data_codewords,
                source_index % self.short_data_codewords,
            )
        } else {
            let long_index = source_index - short_total;
            (
                self.short_block_count + (long_index / self.max_data_codewords),
                long_index % self.max_data_codewords,
            )
        };

        if row < self.short_data_codewords {
            (row * self.block_count) + block
        } else {
            (self.short_data_codewords * self.block_count) + (block - self.short_block_count)
        }
    }

    fn interleave_data_in_place(&self, data: &mut [u8]) {
        let mut visited = new_interleave_visited_buf::<T>();

        for start in 0..data.len() {
            if Self::visited_get(visited.as_ref(), start) {
                continue;
            }

            let mut current = start;
            let mut value = data[current];
            loop {
                Self::visited_set(visited.as_mut(), current);
                let next = self.source_to_interleaved_index(current);
                core::mem::swap(&mut value, &mut data[next]);
                current = next;
                if Self::visited_get(visited.as_ref(), current) {
                    break;
                }
            }
        }
    }

    fn visited_get(visited: &[u8], index: usize) -> bool {
        let byte_index = index >> 3;
        let bit_mask = 1u8 << (index & 0x07);
        (visited[byte_index] & bit_mask) != 0
    }

    fn visited_set(visited: &mut [u8], index: usize) {
        let byte_index = index >> 3;
        let bit_mask = 1u8 << (index & 0x07);
        visited[byte_index] |= bit_mask;
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockLayout, interleave};
    use crate::{
        ecc::ReedSolomon,
        encoder::EncodedData,
        helper::ecc_codewords,
        types::{EccLevel, EncodeMode},
        version::{Version1, Version5, new_data_ecc_buf},
    };

    #[test]
    fn version1_interleave_matches_single_block_append() {
        let mut actual = new_data_ecc_buf::<Version1>();
        let encoded = EncodedData::<Version1>::encode(
            b"HELLO WORLD",
            EncodeMode::Alphanumeric,
            EccLevel::L,
            &mut actual,
        )
        .unwrap();
        let mut expected = actual.clone();
        let ecc_len = ecc_codewords::<Version1>(EccLevel::L);
        let (data, remainder) = expected.as_mut().split_at_mut(encoded.data_len);
        ReedSolomon::generate_ecc_into(&data[..encoded.data_len], &mut remainder[..ecc_len]);

        interleave::<Version1, _>(
            &mut actual,
            encoded,
            EccLevel::L,
            ReedSolomon::generate_ecc_into,
        );
        assert_eq!(actual.as_ref(), expected.as_ref());
    }

    #[test]
    fn block_layout_matches_known_multi_block_distribution() {
        let layout = BlockLayout::<Version5>::new(EccLevel::Q);
        assert_eq!(layout.block_count, 4);
        assert_eq!(layout.short_block_count, 2);
        assert_eq!(layout.short_data_codewords, 15);
        assert_eq!(layout.max_data_codewords, 16);
        assert_eq!(layout.ecc_codewords_per_block, 18);
        assert_eq!(layout.data_codewords(0), 15);
        assert_eq!(layout.data_codewords(1), 15);
        assert_eq!(layout.data_codewords(2), 16);
        assert_eq!(layout.data_codewords(3), 16);
    }
}
