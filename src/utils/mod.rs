// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::borrow::Cow;

use bytesize::ByteSize;

mod error_handling;
pub use error_handling::ValfiskError;
mod nanoid;
pub use nanoid::nanoid;
pub mod serenity;

pub fn truncate(s: &str, new_len: usize) -> String {
    s.chars().take(new_len).collect()
}

pub fn format_bytes(bytes: u64) -> String {
    ByteSize::b(bytes).display().si().to_string()
}

pub fn option_strings<'a>(a: Option<&'a str>, b: Option<&'a str>) -> Option<Cow<'a, str>> {
    a.map_or_else(
        || b.map(Cow::Borrowed),
        |a| {
            b.map_or(Some(Cow::Borrowed(a)), |b| {
                Some(Cow::Owned(a.to_owned() + "\n" + b))
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_strings_works() {
        assert_eq!(option_strings(Some("a"), Some("b")), Some("a\nb".into()));
        assert_eq!(option_strings(Some("a"), None), Some("a".into()));
        assert_eq!(option_strings(None, Some("b")), Some("b".into()));
        assert_eq!(option_strings(None, None), None);
    }
}
