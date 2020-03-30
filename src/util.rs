use crate::traits::Identify;
use crate::{Error, Result};
use crate::{NonNegative, Positive};
use colored::Colorize;
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

/// A builder for lists of things requiring unique names.
///
/// When finish() is called, the names will be checked for uniqueness.
#[derive(Debug)]
pub struct UniqueKeyBuilder<T: Identify> {
    /// Item storage.
    items: Vec<T>,
}

impl<T: Identify> UniqueKeyBuilder<T> {
    /// Construct a new builder.
    pub fn new() -> Self {
        UniqueKeyBuilder::default()
    }

    /// Add an item to the builder.
    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }

    /// Return this builder with the item appended.
    pub fn with_item(mut self, item: T) -> Self {
        self.add_item(item);
        self
    }

    /// Return this builder with the items appended.
    pub fn with_items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        items.into_iter().for_each(|item| {
            self.add_item(item);
        });
        self
    }

    /// Finalize the builder, checking whether all names are unique.
    /// Returns Ok() if names were unique and an Err() otherwise.
    pub fn finish(self) -> Result<Vec<T>> {
        let set: HashSet<&str> = self.items.iter().map(|item| item.identifier()).collect();
        if self.items.len() != set.len() {
            Err(Error::UnexpectedDuplicate)
        } else {
            Ok(self.items)
        }
    }
}

impl<T: Identify> FromIterator<T> for UniqueKeyBuilder<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        UniqueKeyBuilder {
            items: iter.into_iter().collect(),
        }
    }
}

impl<T: Identify> Default for UniqueKeyBuilder<T> {
    fn default() -> Self {
        UniqueKeyBuilder { items: Vec::new() }
    }
}

/// Simple logger for Tydi.
pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{:5}]: {}",
                {
                    let lvl = format!("{}", record.level());
                    let l = lvl.as_str();
                    match record.level() {
                        log::Level::Error => l.red(),
                        log::Level::Warn => l.yellow(),
                        log::Level::Info => l.white(),
                        log::Level::Debug => l.green(),
                        log::Level::Trace => l.bright_black(),
                    }
                },
                record.args()
            );
        }
    }

    fn flush(&self) {}
}
