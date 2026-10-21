#![no_std]

//! VASP Registry Contract
//!
//! Maintains an allowlist of registered Virtual Asset Service Providers (VASPs)
//! that are authorized to participate in travel-rule compliant transfers.
//!
//! Each VASP entry includes:
//! - the VASP's Stellar address (on-chain identity),
//! - an endpoint URL (off-chain service for metadata exchange), and
//! - an `active` flag (soft-delete for audit-trail preservation).
//!
//! Only the contract admin can add, remove, or transfer admin rights.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};
use travel_rule_common::{
    VaspInfo, INSTANCE_BUMP_AMOUNT, INSTANCE_BUMP_THRESHOLD, PERSISTENT_BUMP_AMOUNT,
    PERSISTENT_BUMP_THRESHOLD,
};

/// Storage keys for the contract's state.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// The admin address (instance storage).
    Admin,
    /// VASP info keyed by address (persistent storage).
    Vasp(Address),
}

/// Error codes specific to the registry contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    /// Caller is not the admin.
    Unauthorized = 1,
    /// VASP not found in the registry.
    NotFound = 2,
    /// Contract already initialized.
    AlreadyInitialized = 3,
}

#[contract]
pub struct VaspRegistryContract;

impl VaspRegistryContract {
    /// Load the configured admin, panicking if the contract is uninitialized.
    fn admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized")
    }

    /// Keep the contract configuration entry alive.
    fn bump_instance(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_BUMP_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }

    /// Keep a persistent VASP entry alive.
    fn bump_persistent(env: &Env, key: &DataKey) {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_BUMP_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    /// Authorize `caller` as the admin, returning `Unauthorized` otherwise.
    fn require_admin(env: &Env, caller: &Address) -> Result<(), RegistryError> {
        if caller != &Self::admin(env) {
            return Err(RegistryError::Unauthorized);
        }
        Ok(())
    }
}

#[contractimpl]
impl VaspRegistryContract {
    /// Initialize the VASP registry contract with an admin address.
    ///
    /// # Arguments
    /// * `admin` - the address that will have permission to add/remove VASPs.
    ///
    /// # Authorization
    /// Requires auth from `admin`, preventing a third party from front-running
    /// initialization and taking over the registry.
    ///
    /// # Panics
    /// Panics if the contract has already been initialized.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        Self::bump_instance(&env);
    }

    /// Add a new VASP to the registry, or reactivate/update a removed one.
    ///
    /// # Arguments
    /// * `caller` - must be the admin address.
    /// * `vasp` - the Stellar address of the VASP to register.
    /// * `endpoint_url` - the HTTPS endpoint where this VASP receives metadata.
    ///
    /// # Authorization
    /// Requires auth from `caller`, who must be the admin.
    ///
    /// # Errors
    /// Returns [`RegistryError::Unauthorized`] if `caller` is not the admin.
    pub fn add_vasp(
        env: Env,
        caller: Address,
        vasp: Address,
        endpoint_url: String,
    ) -> Result<(), RegistryError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let info = VaspInfo {
            address: vasp.clone(),
            endpoint_url,
            active: true,
        };

        let key = DataKey::Vasp(vasp.clone());
        env.storage().persistent().set(&key, &info);
        Self::bump_persistent(&env, &key);
        Self::bump_instance(&env);

        env.events().publish((symbol_short!("vasp_add"), vasp), ());
        Ok(())
    }

    /// Remove a VASP from the registry (soft delete - preserves audit trail).
    ///
    /// # Arguments
    /// * `caller` - must be the admin address.
    /// * `vasp` - the address of the VASP to deactivate.
    ///
    /// # Authorization
    /// Requires auth from `caller`, who must be the admin.
    ///
    /// # Errors
    /// Returns [`RegistryError::Unauthorized`] if `caller` is not the admin, or
    /// [`RegistryError::NotFound`] if the VASP was never registered.
    pub fn remove_vasp(env: Env, caller: Address, vasp: Address) -> Result<(), RegistryError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Vasp(vasp.clone());
        let Some(mut info) = env.storage().persistent().get::<_, VaspInfo>(&key) else {
            return Err(RegistryError::NotFound);
        };
        info.active = false;
        env.storage().persistent().set(&key, &info);
        Self::bump_persistent(&env, &key);
        Self::bump_instance(&env);

        env.events().publish((symbol_short!("vasp_rem"), vasp), ());
        Ok(())
    }

    /// Check if an address is a registered and active VASP.
    ///
    /// # Arguments
    /// * `vasp` - the address to check.
    ///
    /// # Returns
    /// `true` if the address is registered and active, `false` otherwise.
    pub fn is_registered(env: Env, vasp: Address) -> bool {
        let key = DataKey::Vasp(vasp);
        let registered = env
            .storage()
            .persistent()
            .get::<_, VaspInfo>(&key)
            .map(|info| info.active)
            .unwrap_or(false);

        // A live lookup is a good opportunity to refresh the entry's TTL.
        if env.storage().persistent().has(&key) {
            Self::bump_persistent(&env, &key);
        }
        Self::bump_instance(&env);
        registered
    }

    /// Get the off-chain endpoint URL for a registered VASP.
    ///
    /// # Arguments
    /// * `vasp` - the address of the VASP.
    ///
    /// # Returns
    /// `Some(url)` if the VASP is registered and active, `None` otherwise.
    pub fn get_endpoint(env: Env, vasp: Address) -> Option<String> {
        Self::bump_instance(&env);
        env.storage()
            .persistent()
            .get::<_, VaspInfo>(&DataKey::Vasp(vasp))
            .filter(|info| info.active)
            .map(|info| info.endpoint_url)
    }

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Address {
        Self::bump_instance(&env);
        Self::admin(&env)
    }

    /// Transfer admin privileges to a new address.
    ///
    /// # Arguments
    /// * `caller` - must be the current admin address.
    /// * `new_admin` - the address that will become the new admin.
    ///
    /// # Authorization
    /// Requires auth from `caller`, who must be the current admin.
    ///
    /// # Errors
    /// Returns [`RegistryError::Unauthorized`] if `caller` is not the admin.
    pub fn transfer_admin(
        env: Env,
        caller: Address,
        new_admin: Address,
    ) -> Result<(), RegistryError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        Self::bump_instance(&env);

        env.events()
            .publish((symbol_short!("admin_xfr"),), (caller, new_admin));
        Ok(())
    }
}

mod test;
