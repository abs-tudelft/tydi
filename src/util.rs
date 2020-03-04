use crate::traits::Identify;
use crate::{Error, Result};
use crate::{NonNegative, Positive};
use log::{Level, Metadata, Record};
use std::collections::HashSet;
use std::iter::FromIterator;

/// Returns ⌈log2(x)⌉.
pub(crate) const fn log2_ceil(x: Positive) -> NonNegative {
    8 * std::mem::size_of::<NonNegative>() as NonNegative
        - (x.get() - 1).leading_zeros() as NonNegative
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log2_ceil_fn() {
        for i in (1..65_536).map(Positive::new).map(Option::unwrap) {
            assert_eq!((i.get() as f64).log2().ceil() as NonNegative, log2_ceil(i));
        }
    }
}

#[derive(Debug)]
pub struct UniquelyNamedBuilder<T: Identify> {
    items: Vec<T>,
}

impl<T: Identify> UniquelyNamedBuilder<T> {
    pub fn new() -> Self {
        UniquelyNamedBuilder::default()
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn with_item(mut self, item: T) -> Self {
        self.add_item(item);
        self
    }

    pub fn with_items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        items.into_iter().for_each(|item| {
            self.add_item(item);
        });
        self
    }

    pub fn finish(self) -> Result<Vec<T>> {
        let set: HashSet<&str> = self.items.iter().map(|item| item.name()).collect();
        if self.items.len() != set.len() {
            Err(Error::UnexpectedDuplicate)
        } else {
            Ok(self.items)
        }
    }
}

impl<T: Identify> FromIterator<T> for UniquelyNamedBuilder<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        UniquelyNamedBuilder {
            items: iter.into_iter().collect(),
        }
    }
}

impl<T: Identify> Default for UniquelyNamedBuilder<T> {
    fn default() -> Self {
        UniquelyNamedBuilder { items: Vec::new() }
    }
}

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{:5} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
