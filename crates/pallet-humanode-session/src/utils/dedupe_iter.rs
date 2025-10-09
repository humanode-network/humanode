//! Deduplicating iterator wrapper.

use alloc::collections::BTreeSet;

/// Dedupe key extraction.
///
/// Provides a dedupe key for type `T`.
pub trait DedupeKeyExtractor<T> {
    /// The type this extractor extracts; the dedupe key.
    type Output;

    /// Perform the dedupe key extraction from the given value.
    fn extract_key(&self, value: &T) -> Self::Output;
}

/// The dedupe iterator.
pub struct DedupeIter<Source, DedupeKeyExtractor>
where
    Source: Iterator,
    DedupeKeyExtractor: self::DedupeKeyExtractor<Source::Item>,
{
    /// The source iterator.
    pub source: Source,

    /// The dedupe key extractor.
    pub dedupe_key_extractor: DedupeKeyExtractor,

    /// The state for tracking the duplicates.
    pub dedupe_state:
        BTreeSet<<DedupeKeyExtractor as self::DedupeKeyExtractor<Source::Item>>::Output>,
}

impl<Source, DedupeKeyExtractor> DedupeIter<Source, DedupeKeyExtractor>
where
    Source: Iterator,
    DedupeKeyExtractor: self::DedupeKeyExtractor<Source::Item>,
{
    /// Create a new dedupe iterator from the given iterator.
    pub fn new(source: Source, dedupe_key_extractor: DedupeKeyExtractor) -> Self {
        Self {
            source,
            dedupe_key_extractor,
            dedupe_state: Default::default(),
        }
    }
}

impl<Source, DedupeKeyExtractor> Iterator for DedupeIter<Source, DedupeKeyExtractor>
where
    Source: Iterator,
    DedupeKeyExtractor: self::DedupeKeyExtractor<Source::Item>,
    DedupeKeyExtractor::Output: Ord,
{
    type Item = Source::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.source.next()?;
            let dedupe_key = self.dedupe_key_extractor.extract_key(&item);
            let was_new = self.dedupe_state.insert(dedupe_key);
            if !was_new {
                continue;
            }
            return Some(item);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_source_lower, source_higher) = self.source.size_hint();

        // Lower bound in always unpredictably `0`.
        (0, source_higher)
    }
}

/// [`Iterator`] extension trait for `dedupe` fn.
pub trait DedupeIteratorExt: Iterator + Sized {
    /// Deduplicate the iterator.
    fn dedupe<DedupeKeyExtractor: self::DedupeKeyExtractor<Self::Item>>(
        self,
        dedupe_key_extractor: DedupeKeyExtractor,
    ) -> DedupeIter<Self, DedupeKeyExtractor>;
}

impl<T> DedupeIteratorExt for T
where
    T: Iterator,
{
    fn dedupe<DedupeKeyExtractor: self::DedupeKeyExtractor<Self::Item>>(
        self,
        dedupe_key_extractor: DedupeKeyExtractor,
    ) -> DedupeIter<Self, DedupeKeyExtractor> {
        DedupeIter::new(self, dedupe_key_extractor)
    }
}

impl<T, R> DedupeKeyExtractor<T> for fn(&T) -> R
where
    R: Eq + core::hash::Hash,
{
    type Output = R;

    fn extract_key(&self, value: &T) -> Self::Output {
        (self)(value)
    }
}
