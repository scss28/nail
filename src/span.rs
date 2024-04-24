use std::ops::Range;

pub type Result<T, E> = std::result::Result<Span<T>, Span<E>>;

pub struct Span<K> {
    kind: K,
    range: Range<usize>,
}

impl<K> Span<K> {}
