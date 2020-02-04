//! Stream-related traits, methods and types.

/// Trait for in-place reversing.
pub trait Reverse {
    fn reverse(&mut self);
}

/// Trait for construction of reversed values.
pub trait Reversed {
    fn reversed(&self) -> Self;
}

impl<T> Reversed for T
where
    T: Reverse + Clone,
{
    fn reversed(&self) -> T {
        let mut r = self.clone();
        r.reverse();
        r
    }
}

/// A direction of flow.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// From source to sink.
    Downstream,
    /// From sink to source.
    Upstream,
}

impl Reverse for Direction {
    fn reverse(&mut self) {
        *self = match self {
            Direction::Downstream => Direction::Upstream,
            Direction::Upstream => Direction::Downstream,
        };
    }
}
