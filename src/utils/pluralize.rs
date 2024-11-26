// SPDX-FileCopyrightText: 2024 Ryan Cao <hello@ryanccn.dev>
//
// SPDX-License-Identifier: AGPL-3.0-only

use num_traits::int::PrimInt as Integer;

/// A trait that simplifies the pluralization of nouns.
pub trait Pluralize {
    /// Pluralize a string by adding `"s"` to the end of it if `count` is not 1.
    #[must_use]
    fn pluralize<T: Integer>(&self, count: T) -> String;
    /// Pluralize a string by returning `alternate` if `count` is not 1.
    #[must_use]
    fn pluralize_alternate<T: Integer>(&self, count: T, alternate: &str) -> String;
}

impl<S> Pluralize for S
where
    S: AsRef<str>,
{
    fn pluralize<T: Integer>(&self, count: T) -> String {
        let mut alternate = self.as_ref().to_string();
        alternate.push('s');
        self.pluralize_alternate(count, &alternate)
    }

    fn pluralize_alternate<T: Integer>(&self, count: T, alternate: &str) -> String {
        if count.is_one() {
            self.as_ref().to_string()
        } else {
            alternate.to_string()
        }
    }
}
