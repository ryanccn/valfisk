// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use eyre::eyre;

struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl<'a> BitReader<'a> {
    const fn new(data: &'a [u8]) -> Self {
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

    // `num_entries` comes off the wire; every entry needs at least one bit.
    let mut values = Vec::with_capacity((num_entries as usize).min(data.len() * 8) + 1);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(values: &[u32], k: u32) -> Vec<u8> {
        let (mut data, mut bit_pos) = (Vec::new(), 0usize);

        let mut write_bit = |bit: bool| {
            if bit_pos % 8 == 0 {
                data.push(0);
            }
            if bit {
                data[bit_pos / 8] |= 1 << (bit_pos % 8);
            }
            bit_pos += 1;
        };

        for delta in values.windows(2).map(|w| w[1] - w[0]) {
            for _ in 0..delta >> k {
                write_bit(true);
            }
            write_bit(false);

            for i in 0..k {
                write_bit(delta >> i & 1 != 0);
            }
        }

        data
    }

    #[test]
    fn decode_round_trips() {
        let values = [10, 15, 100, 1000, 1001, 70000];

        for k in 0..8 {
            let decoded = decode(
                u64::from(values[0]),
                k,
                u32::try_from(values.len() - 1).unwrap(),
                &encode(&values, k),
            )
            .unwrap();

            assert_eq!(decoded, values, "k = {k}");
        }
    }

    #[test]
    fn decode_without_entries_yields_the_first_value() {
        assert_eq!(decode(42, 2, 0, &[]).unwrap(), [42]);
    }

    #[test]
    fn decode_rejects_truncated_data() {
        assert!(decode(0, 5, 100, &[0xff, 0xff]).is_err());
    }

    #[test]
    fn decode_does_not_over_reserve_on_a_large_entry_count() {
        assert!(decode(0, 5, u32::MAX, &[0x00]).is_err());
    }
}
