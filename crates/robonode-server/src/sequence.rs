//! Sequence implementation.

/// An increment-only sequence.
#[derive(Debug)]
pub struct Sequence(u64);

impl Sequence {
    /// Create a new sequence with the specified initial value.
    pub fn new(init: u64) -> Self {
        Self(init)
    }

    /// Increment the sequence value.
    pub fn inc(&mut self) {
        // u64 is big enough for this oveflow to be practicly impossible.
        self.0 = self.0.checked_add(1).unwrap();
    }

    /// Obtain the current value of the sequence.
    pub fn get(&self) -> u64 {
        self.0
    }
}
