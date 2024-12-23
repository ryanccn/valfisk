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

impl Pluralize for String {
    fn pluralize<T: Integer>(&self, count: T) -> String {
        self.pluralize_alternate(count, &(self.clone() + "s"))
    }
    fn pluralize_alternate<T: Integer>(&self, count: T, alternate: &str) -> String {
        if count.is_one() {
            self.clone()
        } else {
            alternate.to_owned()
        }
    }
}

impl Pluralize for &str {
    fn pluralize<T: Integer>(&self, count: T) -> String {
        self.to_owned().to_owned().pluralize(count)
    }
    fn pluralize_alternate<T: Integer>(&self, count: T, alternate: &str) -> String {
        self.to_owned()
            .to_owned()
            .pluralize_alternate(count, alternate)
    }
}
