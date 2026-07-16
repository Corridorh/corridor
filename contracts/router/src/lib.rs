#![no_std]

//! Router Contract
//! 
//! Wraps a SEP-41 token contract and enforces FATF Travel-Rule compliance
//! for transfers at or above a configurable threshold.
//! 
//! Below-threshold transfers pass through directly to the underlying token.
//! At-or-above threshold transfers require:
//! - A cryptographic commitment (metadata_hash) to off-chain compliance data
//! - Both parties must be registered, active VASPs
//! - The hash must not have been used before (replay protection)
//! 
//! **Critical security property:** NO personally identifiable information (PII)
//! is ever stored on-chain. Only SHA-256 hashes and VASP routing metadata.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env, Map,
};
use travel_rule_common::RouterError;

/// Storage keys for the router contract's data
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// The admin address (instance storage)
    Admin,
    /// The wrapped SEP-41 token contract address (instance storage)
    TokenAddress,
    /// The VASP registry contract address (instance storage)
    RegistryAddress,
    /// The amount threshold for compliance checks (instance storage)
    Threshold,
    /// Map of used metadata hashes for replay protection (persistent storage)
    UsedHash(BytesN<32>),
}

#[contract]
pub struct RouterContract;

#[contractimpl]
impl RouterContract {
    /// Initialize the router contract.
    /// 
    /// # Arguments
    /// * `admin` - Address with permission to update the threshold
    /// * `token` - Address of the SEP-41 token contract to wrap
    /// * `registry` - Address of the VASP registry contract
    /// * `threshold` - Minimum amount that triggers compliance checks
    /// 
    /// # Panics
    /// Panics if the contract has already been initialized.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        registry: Address,
        threshold: i128,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage()
            .instance()
            .set(&DataKey::RegistryAddress, &registry);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
    }

    /// Execute a travel-rule compliant token transfer.
    /// 
    /// # Behavior
    /// - If `amount < threshold`: passes through to underlying token immediately
    /// - If `amount >= threshold`: enforces compliance checks before transfer
    /// 
    /// # Compliance checks (when amount >= threshold)
    /// 1. `metadata_hash` must be provided
    /// 2. Hash must not have been used before (replay protection)
    /// 3. Both `from` and `to` must be registered, active VASPs
    /// 
    /// # Arguments
    /// * `from` - The sender's address (must authorize the call)
    /// * `to` - The recipient's address
    /// * `amount` - The transfer amount (in token's smallest unit)
    /// * `metadata_hash` - Optional SHA-256 hash of the off-chain compliance payload
    /// 
    /// # Authorization
    /// Requires auth from the `from` address.
    /// 
    /// # Errors
    /// Returns `RouterError::MissingMetadata` if amount >= threshold and no hash provided
    /// Returns `RouterError::HashAlreadyUsed` if the hash was used in a previous transfer
    /// Returns `RouterError::UnregisteredCounterparty` if either party is not a registered VASP
    /// 
    /// # Events
    /// Emits `compliant_transfer_executed` on success for above-threshold transfers
    pub fn compliant_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        metadata_hash: Option<BytesN<32>>,
    ) -> Result<(), RouterError> {
        from.require_auth();

        let threshold: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .expect("Contract not initialized");

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .expect("Contract not initialized");

        // Below threshold: pass through directly
        if amount < threshold {
            let token_client = token::TokenClient::new(&env, &token_address);
            token_client.transfer(&from, &to, &amount);
            return Ok(());
        }

        // At or above threshold: enforce compliance
        let hash = metadata_hash.ok_or(RouterError::MissingMetadata)?;

        // Check for hash replay
        let hash_key = DataKey::UsedHash(hash.clone());
        if env.storage().persistent().has(&hash_key) {
            return Err(RouterError::HashAlreadyUsed);
        }

        // Verify both parties are registered VASPs
        // Note: In this MVP, we assume `from` and `to` are the VASP custody addresses themselves.
        // A production system would map end-user addresses to their custodian VASPs.
        let registry_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("Contract not initialized");

        // Import the VASP registry client interface
        // We'll use the generated client to call is_registered
        let is_from_registered = Self::check_vasp_registered(&env, &registry_address, &from);
        let is_to_registered = Self::check_vasp_registered(&env, &registry_address, &to);

        if !is_from_registered || !is_to_registered {
            return Err(RouterError::UnregisteredCounterparty);
        }

        // Record the hash as used
        env.storage().persistent().set(&hash_key, &true);

        // Execute the underlying token transfer
        let token_client = token::TokenClient::new(&env, &token_address);
        token_client.transfer(&from, &to, &amount);

        // Emit success event
        env.events().publish(
            (
                symbol_short!("cmp_xfer"),
                from.clone(),
                to.clone(),
            ),
            (amount, hash),
        );

        Ok(())
    }

    /// Update the compliance threshold.
    /// 
    /// # Arguments
    /// * `caller` - Must be the admin address
    /// * `new_threshold` - The new minimum amount for compliance checks
    /// 
    /// # Authorization
    /// Requires auth from the caller, who must be the admin.
    /// 
    /// # Panics
    /// Panics if caller is not the admin.
    pub fn set_threshold(env: Env, caller: Address, new_threshold: i128) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");

        if caller != admin {
            panic!("Unauthorized: only admin can set threshold");
        }

        env.storage()
            .instance()
            .set(&DataKey::Threshold, &new_threshold);
    }

    /// Get the current compliance threshold.
    /// 
    /// # Returns
    /// The minimum transfer amount that triggers compliance checks.
    pub fn get_threshold(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Threshold)
            .expect("Contract not initialized")
    }

    // Helper function to check VASP registration
    // In a real implementation, this would use the generated client from vasp-registry
    fn check_vasp_registered(env: &Env, registry: &Address, vasp: &Address) -> bool {
        // Call the registry's is_registered function via contract invocation
        env.invoke_contract::<bool>(
            registry,
            &soroban_sdk::symbol_short!("is_registered"),
            soroban_sdk::vec![env, vasp.into_val(env)],
        )
    }
}

mod test;
