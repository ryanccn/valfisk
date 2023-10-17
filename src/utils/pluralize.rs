use num::Integer;

pub trait Pluralize {
    fn pluralize<T: Integer>(&self, count: T) -> String;
}

impl Pluralize for String {
    fn pluralize<T: Integer>(&self, count: T) -> String {
        self.to_owned() + if count.is_one() { "" } else { "s" }
    }
}

impl Pluralize for &str {
    fn pluralize<T: Integer>(&self, count: T) -> String {
        self.to_owned().to_owned().pluralize(count)
    }
}
