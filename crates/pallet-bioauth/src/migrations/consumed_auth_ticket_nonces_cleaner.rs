//! Migration to clean consumed auth ticket nonces.

/// Execute migration to clean consumed auth ticket nonces.
pub struct ConsumedAuthTicketNoncesCleaner<T>(sp_std::marker::PhantomData<T>);
