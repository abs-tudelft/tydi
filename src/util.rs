use crate::traits::Name;
use crate::{Error, Result};
use crate::{NonNegative, Positive};
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

pub struct UniquelyNamedBuilder<T: Name> {
    items: Vec<T>,
}

impl<T: Name> UniquelyNamedBuilder<T> {
    pub fn new() -> Self {
        UniquelyNamedBuilder { items: Vec::new() }
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

impl<T: Name> FromIterator<T> for UniquelyNamedBuilder<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        UniquelyNamedBuilder {
            items: iter.into_iter().collect(),
        }
    }
}
