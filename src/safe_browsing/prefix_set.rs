// SPDX-FileCopyrightText: 2026 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::cmp::Ordering;

/// A lexicographically sorted set of hash prefixes packed into one allocation.
///
/// Lists hold up to 100 000 prefixes of 4 to 32 bytes; a `Vec<Vec<u8>>` would spend
/// more memory on allocator bookkeeping than on the prefixes themselves.
#[derive(Debug, Clone, Default)]
pub struct PrefixSet {
    data: Vec<u8>,
    /// Prefix `i` spans `offsets[i]..offsets[i + 1]`.
    offsets: Vec<usize>,
    /// The distinct prefix lengths present, ascending.
    lengths: Vec<usize>,
}

impl PrefixSet {
    pub fn build(mut prefixes: Vec<&[u8]>) -> Self {
        prefixes.sort_unstable();

        let mut data = Vec::with_capacity(prefixes.iter().map(|p| p.len()).sum());
        let mut offsets = Vec::with_capacity(prefixes.len() + 1);
        let mut lengths = Vec::new();

        offsets.push(0);

        for prefix in prefixes {
            data.extend_from_slice(prefix);
            offsets.push(data.len());

            if !lengths.contains(&prefix.len()) {
                lengths.push(prefix.len());
            }
        }

        lengths.sort_unstable();

        Self {
            data,
            offsets,
            lengths,
        }
    }

    pub fn len(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The concatenation the list checksum is computed over.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn get(&self, index: usize) -> &[u8] {
        &self.data[self.offsets[index]..self.offsets[index + 1]]
    }

    pub fn contains(&self, prefix: &[u8]) -> bool {
        if self.is_empty() {
            return false;
        }

        let (mut lo, mut hi) = (0, self.len());

        while lo < hi {
            let mid = lo + (hi - lo) / 2;

            match self.get(mid).cmp(prefix) {
                Ordering::Less => lo = mid + 1,
                Ordering::Greater => hi = mid,
                Ordering::Equal => return true,
            }
        }

        false
    }

    pub fn matching_prefix<'a>(&self, hash: &'a [u8]) -> Option<&'a [u8]> {
        self.lengths
            .iter()
            .filter_map(|&len| hash.get(..len))
            .find(|prefix| self.contains(prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(set: &PrefixSet) -> Vec<&[u8]> {
        (0..set.len()).map(|i| set.get(i)).collect()
    }

    #[test]
    fn empty_set_matches_nothing() {
        let set = PrefixSet::build(Vec::new());

        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
        assert!(set.as_bytes().is_empty());
        assert!(!set.contains(b"abcd"));
        assert_eq!(set.matching_prefix(b"abcdefgh"), None);
    }

    #[test]
    fn build_sorts_and_packs() {
        let set = PrefixSet::build(vec![b"ccc", b"aaa", b"bbb"]);

        assert_eq!(set.len(), 3);
        assert_eq!(set.as_bytes(), b"aaabbbccc");
        assert_eq!(collect(&set), [b"aaa", b"bbb", b"ccc"]);
    }

    #[test]
    fn contains_finds_every_member() {
        let prefixes: Vec<[u8; 4]> = (0u32..500).map(|i| i.to_le_bytes()).collect();
        let set = PrefixSet::build(prefixes.iter().map(<[u8; 4]>::as_slice).collect());

        assert_eq!(set.len(), 500);

        for prefix in &prefixes {
            assert!(set.contains(prefix));
        }

        assert!(!set.contains(&500u32.to_le_bytes()));
        assert!(!set.contains(b"\xff\xff\xff\xff"));
    }

    #[test]
    fn matching_prefix_handles_mixed_lengths() {
        let set = PrefixSet::build(vec![b"abcd", b"wxyz012345"]);

        assert_eq!(set.matching_prefix(b"abcdefghij"), Some(&b"abcd"[..]));
        assert_eq!(set.matching_prefix(b"wxyz012345"), Some(&b"wxyz012345"[..]));
        // Present only at the longer length, so a short hash must not match.
        assert_eq!(set.matching_prefix(b"wxyz01"), None);
        assert_eq!(set.matching_prefix(b"nomatch___"), None);
    }

    #[test]
    fn matching_prefix_ignores_hashes_shorter_than_the_prefix() {
        let set = PrefixSet::build(vec![b"abcd"]);
        assert_eq!(set.matching_prefix(b"abc"), None);
    }
}
