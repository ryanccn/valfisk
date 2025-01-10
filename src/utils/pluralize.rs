// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use num_traits::int::PrimInt as Integer;

/// A trait that simplifies the pluralization of nouns.
pub trait Pluralize {
    /// Pluralize a string by adding `"s"` to the end of it if `count` is not 1.
    #[must_use]
    fn pluralize<T: Integer>(&self, count: T) -> String;

    /// Pluralize a string by adding a suffix to the end of it if `count` is not 1.
    #[must_use]
    fn pluralize_suffix<T: Integer, F: AsRef<str>>(&self, count: T, suffix: F) -> String;

    /// Pluralize a string by returning `alternate` if `count` is not 1.
    #[must_use]
    fn pluralize_alternate<T: Integer, F: AsRef<str>>(&self, count: T, alternate: F) -> String;
}

impl<S> Pluralize for S
where
    S: AsRef<str>,
{
    fn pluralize<T: Integer>(&self, count: T) -> String {
        self.pluralize_suffix(count, "s")
    }

    fn pluralize_suffix<T: Integer, F: AsRef<str>>(&self, count: T, suffix: F) -> String {
        let mut alternate = self.as_ref().to_owned();
        alternate.push_str(suffix.as_ref());

        self.pluralize_alternate(count, &alternate)
    }

    fn pluralize_alternate<T: Integer, F: AsRef<str>>(&self, count: T, alternate: F) -> String {
        if count.is_one() {
            self.as_ref().to_owned()
        } else {
            alternate.as_ref().to_owned()
        }
    }
}
