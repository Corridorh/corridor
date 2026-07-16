#![no_std]

/// Shared types and error codes for the Travel Rule compliance system.
/// 
/// This crate provides common functionality used by both the router and
/// vasp-registry contracts to ensure consistency across the system.

use soroban_sdk::{contracttype, Address, String};

/// Information about a registered VASP (Virtual Asset Service Provider).
/// 
/// This struct stores the identity and connectivity information for a VASP
/// that is authorized to participate in compliant transfers. The endpoint_url
/// is the off-chain service URL where this VASP receives travel rule metadata.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaspInfo {
    /// The Stellar address that identifies this VASP on-chain
    pub address: Address,
    /// Off-chain HTTPS endpoint for receiving travel rule metadata
    /// Example: "https://vasp-a.example.com/travelrule"
    pub endpoint_url: String,
    /// Whether this VASP is currently active (soft-delete flag)
    pub active: bool,
}

/// Error codes for the router contract.
/// 
/// These errors are returned when the compliance checks fail or when
/// invalid operations are attempted on the router contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RouterError {
    /// Required metadata_hash was not provided for a transfer at or above threshold
    MissingMetadata = 1,
    /// The provided metadata_hash has already been used (replay attack prevention)
    HashAlreadyUsed = 2,
    /// One or both parties in the transfer are not registered, active VASPs
    UnregisteredCounterparty = 3,
    /// Caller is not authorized to perform this operation
    Unauthorized = 4,
}
