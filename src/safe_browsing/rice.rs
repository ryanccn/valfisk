// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::eyre;

struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, bit_pos: 0 }
    }

    fn read_bit(&mut self) -> Option<bool> {
        let byte_idx = self.bit_pos / 8;
        let bit_idx = self.bit_pos % 8;
        if byte_idx >= self.data.len() {
            return None;
        }
        let bit = (self.data[byte_idx] >> bit_idx) & 1 != 0;
        self.bit_pos += 1;
        Some(bit)
    }

    fn read_bits(&mut self, n: u32) -> Option<u64> {
        let mut result = 0u64;
        for i in 0..n {
            let bit = u64::from(self.read_bit()?);
            result |= bit << i;
        }
        Some(result)
    }

    fn read_golomb_rice(&mut self, k: u32) -> Option<u64> {
        let mut q = 0u64;
        loop {
            if self.read_bit()? {
                q += 1;
            } else {
                break;
            }
        }
        let r = self.read_bits(k)?;
        Some((q << k) | r)
    }
}

pub fn decode(
    first_value: u64,
    rice_parameter: u32,
    num_entries: u32,
    data: &[u8],
) -> eyre::Result<Vec<u32>> {
    let mut reader = BitReader::new(data);
    let mut values = Vec::with_capacity(num_entries as usize + 1);

    #[expect(clippy::cast_possible_truncation)]
    let first = first_value as u32;
    values.push(first);

    let mut prev = first;
    for _ in 0..num_entries {
        #[expect(clippy::cast_possible_truncation)]
        let delta = reader
            .read_golomb_rice(rice_parameter)
            .ok_or_else(|| eyre!("unexpected end of Rice-encoded data"))?
            as u32;
        prev = prev.wrapping_add(delta);
        values.push(prev);
    }

    Ok(values)
}
