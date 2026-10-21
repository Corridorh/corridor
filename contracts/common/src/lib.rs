#![no_std]

//! Shared types, error codes, and storage-lifetime constants for the Travel
//! Rule compliance system.
//!
//! This crate provides the common functionality used by both the `router` and
//! `vasp-registry` contracts so that the on-chain data model stays consistent
//! across the system.

use soroban_sdk::{contracterror, contracttype, Address, String};

/// Approximate number of ledgers in a day (5 second close time).
///
/// Used to express storage lifetimes in human-readable units. The values are
/// deliberately conservative; Soroban caps entry TTLs at the network maximum,
/// so requesting a longer lifetime than the network allows is safe.
pub const DAY_IN_LEDGERS: u32 = 17_280;

/// Bump instance (contract configuration) storage when its TTL falls below
/// this threshold (≈30 days).
pub const INSTANCE_BUMP_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;

/// Extend instance storage back up to this lifetime (≈60 days).
pub const INSTANCE_BUMP_AMOUNT: u32 = 60 * DAY_IN_LEDGERS;

/// Bump a persistent entry when its TTL falls below this threshold (≈30 days).
pub const PERSISTENT_BUMP_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;

/// Extend a persistent entry back up to this lifetime (≈90 days).
pub const PERSISTENT_BUMP_AMOUNT: u32 = 90 * DAY_IN_LEDGERS;

// Compile-time invariants: a bump amount must always exceed its threshold,
// otherwise `extend_ttl` could shorten an entry's lifetime.
const _: () = assert!(INSTANCE_BUMP_THRESHOLD < INSTANCE_BUMP_AMOUNT);
const _: () = assert!(PERSISTENT_BUMP_THRESHOLD < PERSISTENT_BUMP_AMOUNT);

/// Information about a registered VASP (Virtual Asset Service Provider).
///
/// This struct stores the identity and connectivity information for a VASP
/// that is authorized to participate in compliant transfers. The
/// `endpoint_url` is the off-chain service URL where this VASP receives travel
/// rule metadata.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaspInfo {
    /// The Stellar address that identifies this VASP on-chain.
    pub address: Address,
    /// Off-chain HTTPS endpoint for receiving travel rule metadata.
    /// Example: `https://vasp-a.example.com/travelrule`
    pub endpoint_url: String,
    /// Whether this VASP is currently active (soft-delete flag).
    pub active: bool,
}

/// Error codes returned by the router contract.
///
/// These errors are returned when the compliance checks fail or when invalid
/// operations are attempted on the router contract. They are encoded as
/// `u32` values in the contract's `Result` return type.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RouterError {
    /// Required `metadata_hash` was not provided for a transfer at or above
    /// the compliance threshold.
    MissingMetadata = 1,
    /// The provided `metadata_hash` has already been used (replay prevention).
    HashAlreadyUsed = 2,
    /// One or both parties in the transfer are not registered, active VASPs.
    UnregisteredCounterparty = 3,
    /// Caller is not authorized to perform this operation.
    Unauthorized = 4,
    /// The transfer amount was zero or negative.
    InvalidAmount = 5,
    /// The configured threshold was negative.
    InvalidThreshold = 6,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn router_error_codes_are_stable() {
        // These numeric values are part of the public ABI and must not change.
        let codes = [
            (RouterError::MissingMetadata, 1u32),
            (RouterError::HashAlreadyUsed, 2u32),
            (RouterError::UnregisteredCounterparty, 3u32),
            (RouterError::Unauthorized, 4u32),
            (RouterError::InvalidAmount, 5u32),
            (RouterError::InvalidThreshold, 6u32),
        ];
        for (error, expected) in codes {
            assert_eq!(error as u32, expected);
        }
    }
}
