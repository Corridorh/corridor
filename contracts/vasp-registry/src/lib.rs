#![no_std]

//! VASP Registry Contract
//! 
//! Maintains an allowlist of registered Virtual Asset Service Providers (VASPs)
//! that are authorized to participate in travel-rule compliant transfers.
//! 
//! Each VASP entry includes:
//! - The VASP's Stellar address (on-chain identity)
//! - An endpoint URL (off-chain service for metadata exchange)
//! - An active flag (soft-delete for audit trail preservation)
//! 
//! Only the contract admin can add or remove VASPs.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};
use travel_rule_common::VaspInfo;

/// Storage keys for the contract's persistent data
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// The admin address (instance storage)
    Admin,
    /// VASP info keyed by address (persistent storage)
    Vasp(Address),
}

/// Error codes specific to the registry contract
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    /// Caller is not the admin
    Unauthorized = 1,
    /// VASP not found in registry
    NotFound = 2,
    /// Contract already initialized
    AlreadyInitialized = 3,
}

#[contract]
pub struct VaspRegistryContract;

#[contractimpl]
impl VaspRegistryContract {
    /// Initialize the VASP registry contract with an admin address.
    /// 
    /// # Arguments
    /// * `admin` - The address that will have permission to add/remove VASPs
    /// 
    /// # Panics
    /// Panics if the contract has already been initialized.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Add a new VASP to the registry or reactivate a removed one.
    /// 
    /// # Arguments
    /// * `caller` - Must be the admin address
    /// * `vasp` - The Stellar address of the VASP to register
    /// * `endpoint_url` - The HTTPS endpoint where this VASP receives metadata
    /// 
    /// # Authorization
    /// Requires auth from the caller, who must be the admin.
    /// 
    /// # Panics
    /// Panics if caller is not the admin.
    pub fn add_vasp(env: Env, caller: Address, vasp: Address, endpoint_url: String) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");

        if caller != admin {
            panic!("Unauthorized: only admin can add VASPs");
        }

        let vasp_info = VaspInfo {
            address: vasp.clone(),
            endpoint_url,
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Vasp(vasp.clone()), &vasp_info);

        env.events()
            .publish((symbol_short!("vasp_add"), vasp), ());
    }

    /// Remove a VASP from the registry (soft delete - preserves audit trail).
    /// 
    /// # Arguments
    /// * `caller` - Must be the admin address
    /// * `vasp` - The address of the VASP to deactivate
    /// 
    /// # Authorization
    /// Requires auth from the caller, who must be the admin.
    /// 
    /// # Panics
    /// Panics if caller is not the admin.
    pub fn remove_vasp(env: Env, caller: Address, vasp: Address) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");

        if caller != admin {
            panic!("Unauthorized: only admin can remove VASPs");
        }

        let key = DataKey::Vasp(vasp.clone());
        if let Some(mut vasp_info) = env.storage().persistent().get::<_, VaspInfo>(&key) {
            vasp_info.active = false;
            env.storage().persistent().set(&key, &vasp_info);

            env.events()
                .publish((symbol_short!("vasp_rem"), vasp), ());
        }
    }

    /// Check if an address is a registered and active VASP.
    /// 
    /// # Arguments
    /// * `vasp` - The address to check
    /// 
    /// # Returns
    /// `true` if the address is registered and active, `false` otherwise.
    pub fn is_registered(env: Env, vasp: Address) -> bool {
        env.storage()
            .persistent()
            .get::<_, VaspInfo>(&DataKey::Vasp(vasp))
            .map(|info| info.active)
            .unwrap_or(false)
    }

    /// Get the off-chain endpoint URL for a registered VASP.
    /// 
    /// # Arguments
    /// * `vasp` - The address of the VASP
    /// 
    /// # Returns
    /// `Some(url)` if the VASP is registered and active, `None` otherwise.
    pub fn get_endpoint(env: Env, vasp: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get::<_, VaspInfo>(&DataKey::Vasp(vasp))
            .filter(|info| info.active)
            .map(|info| info.endpoint_url)
    }

    /// Transfer admin privileges to a new address.
    /// 
    /// # Arguments
    /// * `caller` - Must be the current admin address
    /// * `new_admin` - The address that will become the new admin
    /// 
    /// # Authorization
    /// Requires auth from the caller, who must be the current admin.
    /// 
    /// # Panics
    /// Panics if caller is not the current admin.
    pub fn transfer_admin(env: Env, caller: Address, new_admin: Address) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");

        if caller != admin {
            panic!("Unauthorized: only admin can transfer admin");
        }

        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }
}

mod test;
