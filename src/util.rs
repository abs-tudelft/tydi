use crate::error::Error;
use indexmap::IndexMap;
use std::{collections::HashSet, error, num::NonZeroUsize};

/// Returns ⌈log2(x)⌉.
pub(crate) const fn log2_ceil(x: NonZeroUsize) -> usize {
    8 * std::mem::size_of::<usize>() - (x.get() - 1).leading_zeros() as usize
}

/// Returns an `IndexMap` from the provided iterator. Returns an error when
/// there exist duplicate keys. Key matching is case-sensitive.
pub(crate) fn unique_index_map<T>(
    iter: impl IntoIterator<Item = (Option<impl Into<String>>, T)>,
) -> Result<IndexMap<Option<String>, T>, Box<dyn error::Error>> {
    let elements = iter
        .into_iter()
        .map(|(k, v)| (k.map(|n| n.into()), v))
        .collect::<Vec<(Option<String>, T)>>();
    let mut set = HashSet::new();
    if !elements.iter().map(|(k, _)| k).all(|k| set.insert(k)) {
        Err(Box::new(Error::UnexpectedDuplicate))
    } else {
        Ok(elements.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log2_ceil_fn() {
        for i in (1..65_536).map(NonZeroUsize::new).map(Option::unwrap) {
            assert_eq!((i.get() as f64).log2().ceil() as usize, log2_ceil(i));
        }
    }

    #[test]
    fn unique_index_map_fn() -> Result<(), Box<dyn error::Error>> {
        let elements = vec![(None, ()), (Some("asdf".to_string()), ())];
        assert_eq!(
            unique_index_map(elements.clone())?,
            elements
                .into_iter()
                .collect::<IndexMap<Option<String>, ()>>()
        );

        let elements = vec![(None, ()), (None, ()), (Some("a"), ())];
        assert_eq!(
            unique_index_map(elements).unwrap_err().to_string(),
            "Unexpected duplicate"
        );

        Ok(())
    }
}
