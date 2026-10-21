#![no_std]

//! Router Contract
//!
//! Wraps a SEP-41 token contract and enforces FATF Travel-Rule compliance for
//! transfers at or above a configurable threshold.
//!
//! Below-threshold transfers pass through directly to the underlying token.
//! At-or-above threshold transfers require:
//! - a cryptographic commitment (`metadata_hash`) to off-chain compliance data,
//! - both parties to be registered, active VASPs, and
//! - the hash to be unused (replay protection).
//!
//! **Critical security property:** NO personally identifiable information (PII)
//! is ever stored on-chain. Only SHA-256 hashes and VASP routing metadata.

use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env,
};
use travel_rule_common::{
    RouterError, INSTANCE_BUMP_AMOUNT, INSTANCE_BUMP_THRESHOLD, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_BUMP_THRESHOLD,
};

/// The subset of the VASP registry interface the router depends on.
///
/// Declaring it as a `#[contractclient]` gives us a typed, checked call
/// instead of hand-rolling a `Symbol` for `Env::invoke_contract`.
#[contractclient(name = "VaspRegistryClient")]
pub trait VaspRegistryInterface {
    fn is_registered(env: Env, vasp: Address) -> bool;
}

/// Storage keys for the router contract's state.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// The admin address (instance storage).
    Admin,
    /// The wrapped SEP-41 token contract address (instance storage).
    TokenAddress,
    /// The VASP registry contract address (instance storage).
    RegistryAddress,
    /// The amount threshold for compliance checks (instance storage).
    Threshold,
    /// Map of used metadata hashes for replay protection (persistent storage).
    UsedHash(BytesN<32>),
}

#[contract]
pub struct RouterContract;

#[contractimpl]
impl RouterContract {
    /// Initialize the router contract.
    ///
    /// # Arguments
    /// * `admin` - address with permission to update the threshold.
    /// * `token` - address of the SEP-41 token contract to wrap.
    /// * `registry` - address of the VASP registry contract.
    /// * `threshold` - minimum amount that triggers compliance checks.
    ///
    /// # Authorization
    /// Requires auth from `admin`, preventing a third party from front-running
    /// initialization and installing themselves as admin.
    ///
    /// # Panics
    /// Panics if the contract has already been initialized or if `threshold`
    /// is negative.
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
        if threshold < 0 {
            panic!("Threshold must be non-negative");
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage()
            .instance()
            .set(&DataKey::RegistryAddress, &registry);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        Self::bump_instance(&env);
    }

    /// Execute a travel-rule compliant token transfer.
    ///
    /// # Behavior
    /// - If `amount < threshold`: passes through to the underlying token.
    /// - If `amount >= threshold`: enforces compliance checks before transfer.
    ///
    /// # Compliance checks (when `amount >= threshold`)
    /// 1. `metadata_hash` must be provided.
    /// 2. The hash must not have been used before (replay protection).
    /// 3. Both `from` and `to` must be registered, active VASPs.
    ///
    /// # Arguments
    /// * `from` - the sender's address (must authorize the call).
    /// * `to` - the recipient's address.
    /// * `amount` - the transfer amount (in the token's smallest unit).
    /// * `metadata_hash` - optional SHA-256 hash of the off-chain payload.
    ///
    /// # Authorization
    /// Requires auth from the `from` address.
    ///
    /// # Errors
    /// - [`RouterError::InvalidAmount`] if `amount` is zero or negative.
    /// - [`RouterError::MissingMetadata`] if `amount >= threshold` and no hash
    ///   was provided.
    /// - [`RouterError::HashAlreadyUsed`] if the hash was used previously.
    /// - [`RouterError::UnregisteredCounterparty`] if either party is not an
    ///   active VASP.
    ///
    /// # Events
    /// Emits `cmp_xfer` (topics: `from`, `to`) with `(amount, hash)` on a
    /// compliant transfer, and `xfer` (topics: `from`, `to`) with `amount` on a
    /// below-threshold pass-through.
    pub fn compliant_transfer(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        metadata_hash: Option<BytesN<32>>,
    ) -> Result<(), RouterError> {
        from.require_auth();

        if amount <= 0 {
            return Err(RouterError::InvalidAmount);
        }

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
        Self::bump_instance(&env);

        // Below threshold: pass through directly.
        if amount < threshold {
            token::Client::new(&env, &token_address).transfer(&from, &to, &amount);
            env.events()
                .publish((symbol_short!("xfer"), from, to), amount);
            return Ok(());
        }

        // At or above threshold: enforce compliance.
        let hash = metadata_hash.ok_or(RouterError::MissingMetadata)?;

        // Replay protection.
        let hash_key = DataKey::UsedHash(hash.clone());
        if env.storage().persistent().has(&hash_key) {
            return Err(RouterError::HashAlreadyUsed);
        }

        // Verify both parties are registered, active VASPs.
        let registry_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("Contract not initialized");
        let registry = VaspRegistryClient::new(&env, &registry_address);
        if !registry.is_registered(&from) || !registry.is_registered(&to) {
            return Err(RouterError::UnregisteredCounterparty);
        }

        // Effects before interaction: record the hash before moving funds so a
        // re-entrant token cannot replay it.
        env.storage().persistent().set(&hash_key, &true);
        env.storage().persistent().extend_ttl(
            &hash_key,
            PERSISTENT_BUMP_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        token::Client::new(&env, &token_address).transfer(&from, &to, &amount);

        env.events()
            .publish((symbol_short!("cmp_xfer"), from, to), (amount, hash));

        Ok(())
    }

    /// Update the compliance threshold.
    ///
    /// # Arguments
    /// * `caller` - must be the admin address.
    /// * `new_threshold` - the new minimum amount for compliance checks.
    ///
    /// # Authorization
    /// Requires auth from the caller, who must be the admin.
    ///
    /// # Errors
    /// Returns [`RouterError::Unauthorized`] if `caller` is not the admin, or
    /// [`RouterError::InvalidThreshold`] if `new_threshold` is negative.
    pub fn set_threshold(
        env: Env,
        caller: Address,
        new_threshold: i128,
    ) -> Result<(), RouterError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");
        if caller != admin {
            return Err(RouterError::Unauthorized);
        }
        if new_threshold < 0 {
            return Err(RouterError::InvalidThreshold);
        }

        env.storage()
            .instance()
            .set(&DataKey::Threshold, &new_threshold);
        Self::bump_instance(&env);
        Ok(())
    }

    /// Get the current compliance threshold.
    pub fn get_threshold(env: Env) -> i128 {
        Self::bump_instance(&env);
        env.storage()
            .instance()
            .get(&DataKey::Threshold)
            .expect("Contract not initialized")
    }

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Address {
        Self::bump_instance(&env);
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized")
    }

    /// Get the wrapped SEP-41 token contract address.
    pub fn get_token(env: Env) -> Address {
        Self::bump_instance(&env);
        env.storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .expect("Contract not initialized")
    }

    /// Get the configured VASP registry contract address.
    pub fn get_registry(env: Env) -> Address {
        Self::bump_instance(&env);
        env.storage()
            .instance()
            .get(&DataKey::RegistryAddress)
            .expect("Contract not initialized")
    }

    /// Check whether a metadata hash has already been consumed.
    pub fn is_hash_used(env: Env, hash: BytesN<32>) -> bool {
        env.storage().persistent().has(&DataKey::UsedHash(hash))
    }

    /// Keep the contract configuration entry alive.
    fn bump_instance(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }
}

mod test;
