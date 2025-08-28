//! EVM gasometer events definitions.

use codec::{Decode, Encode};

/// Snapshot.
#[derive(Debug, Default, Copy, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Snapshot {
    /// Gas limit.
    pub gas_limit: u64,
    /// Memory gas.
    pub memory_gas: u64,
    /// Used gas.
    pub used_gas: u64,
    /// Refunded gas.
    pub refunded_gas: i64,
}

impl Snapshot {
    /// Calculate gas.
    pub fn gas(&self) -> u64 {
        self.gas_limit
            .saturating_sub(self.used_gas)
            .saturating_sub(self.memory_gas)
    }
}

#[cfg(feature = "evm-tracing")]
impl From<Option<evm_gasometer::Snapshot>> for Snapshot {
    fn from(snapshot: Option<evm_gasometer::Snapshot>) -> Self {
        if let Some(snapshot) = snapshot {
            Self {
                gas_limit: snapshot.gas_limit,
                memory_gas: snapshot.memory_gas,
                used_gas: snapshot.used_gas,
                refunded_gas: snapshot.refunded_gas,
            }
        } else {
            Default::default()
        }
    }
}

/// EVM gasometer event.
#[derive(Debug, Copy, Clone, Encode, Decode, PartialEq, Eq)]
pub enum GasometerEvent {
    /// Record cost.
    RecordCost {
        /// Cost.
        cost: u64,
        /// Snapshot.
        snapshot: Snapshot,
    },
    /// Record refund.
    RecordRefund {
        /// Refund.
        refund: i64,
        /// Snapshot.
        snapshot: Snapshot,
    },
    /// Record stipend.
    RecordStipend {
        /// Stipend.
        stipend: u64,
        /// Snapshot.
        snapshot: Snapshot,
    },
    /// Record dynamic cost.
    RecordDynamicCost {
        /// Gas cost.
        gas_cost: u64,
        /// Memory gas.
        memory_gas: u64,
        /// Gas refunded.
        gas_refund: i64,
        /// Snapshot.
        snapshot: Snapshot,
    },
    /// Record transaction.
    RecordTransaction {
        /// Cost.
        cost: u64,
        /// Snapshot.
        snapshot: Snapshot,
    },
}

#[cfg(feature = "evm-tracing")]
impl From<evm_gasometer::tracing::Event> for GasometerEvent {
    fn from(event: evm_gasometer::tracing::Event) -> Self {
        match event {
            evm_gasometer::tracing::Event::RecordCost { cost, snapshot } => Self::RecordCost {
                cost,
                snapshot: snapshot.into(),
            },
            evm_gasometer::tracing::Event::RecordRefund { refund, snapshot } => {
                Self::RecordRefund {
                    refund,
                    snapshot: snapshot.into(),
                }
            }
            evm_gasometer::tracing::Event::RecordStipend { stipend, snapshot } => {
                Self::RecordStipend {
                    stipend,
                    snapshot: snapshot.into(),
                }
            }
            evm_gasometer::tracing::Event::RecordDynamicCost {
                gas_cost,
                memory_gas,
                gas_refund,
                snapshot,
            } => Self::RecordDynamicCost {
                gas_cost,
                memory_gas,
                gas_refund,
                snapshot: snapshot.into(),
            },
            evm_gasometer::tracing::Event::RecordTransaction { cost, snapshot } => {
                Self::RecordTransaction {
                    cost,
                    snapshot: snapshot.into(),
                }
            }
        }
    }
}
