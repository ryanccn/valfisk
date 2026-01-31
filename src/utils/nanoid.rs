// SPDX-FileCopyrightText: 2025 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use rand::distr::{Distribution as _, slice::Choose};

static ALPHABET: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '_', '-',
];

pub fn nanoid(len: usize) -> String {
    Choose::new(&ALPHABET)
        .unwrap()
        .sample_iter(&mut rand::rng())
        .take(len)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nanoid_works() {
        let id = nanoid(12);
        assert_eq!(id.len(), 12);
        assert!(id.chars().all(|c| ALPHABET.contains(&c)));
    }
}
