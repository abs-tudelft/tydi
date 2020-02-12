use crate::{NonNegative, Positive};

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
