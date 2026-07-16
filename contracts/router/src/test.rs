#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    token, Address, BytesN, Env, String,
};

// Import the VASP registry contract for integration tests
use travel_rule_common::VaspInfo;

// Declare the external contracts we'll use in tests
extern crate vasp_registry;
use vasp_registry::{VaspRegistryContract, VaspRegistryContractClient};

// We'll create a simple test token using Soroban's built-in token functionality
fn create_token_contract<'a>(env: &Env) -> (Address, token::Client<'a>) {
    let token_address = env.register_stellar_asset_contract(Address::generate(env));
    (token_address.clone(), token::Client::new(env, &token_address))
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RouterContract);
    let client = RouterContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env);
    let registry = Address::generate(&env);
    let threshold = 1000i128;

    client.initialize(&admin, &token_address, &registry, &threshold);

    assert_eq!(client.get_threshold(), threshold);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RouterContract);
    let client = RouterContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env);
    let registry = Address::generate(&env);
    let threshold = 1000i128;

    client.initialize(&admin, &token_address, &registry, &threshold);
    client.initialize(&admin, &token_address, &registry, &threshold); // Should panic
}

#[test]
fn test_transfer_below_threshold_succeeds_without_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let (token_address, token_client) = create_token_contract(&env);
    
    // Setup registry (not needed for below-threshold, but required for initialization)
    let registry = Address::generate(&env);
    
    // Setup router
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    let threshold = 1000i128;
    router_client.initialize(&admin, &token_address, &registry, &threshold);

    // Setup test accounts
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    // Mint tokens to sender
    token_client.mint(&from, &500i128);
    
    // Transfer below threshold (no metadata required)
    let result = router_client.try_compliant_transfer(&from, &to, &500i128, &None);
    assert!(result.is_ok());
    
    // Verify balance
    assert_eq!(token_client.balance(&to), 500i128);
}

#[test]
fn test_transfer_at_threshold_requires_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env);
    let registry = Address::generate(&env);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    let threshold = 1000i128;
    router_client.initialize(&admin, &token_address, &registry, &threshold);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    // Transfer exactly at threshold without metadata should fail
    let result = router_client.try_compliant_transfer(&from, &to, &1000i128, &None);
    
    assert_eq!(result, Err(Ok(RouterError::MissingMetadata)));
}

#[test]
fn test_transfer_above_threshold_requires_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env);
    let registry = Address::generate(&env);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    let threshold = 1000i128;
    router_client.initialize(&admin, &token_address, &registry, &threshold);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    // Transfer above threshold without metadata should fail
    let result = router_client.try_compliant_transfer(&from, &to, &5000i128, &None);
    
    assert_eq!(result, Err(Ok(RouterError::MissingMetadata)));
}

#[test]
fn test_hash_replay_protection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, token_client) = create_token_contract(&env);
    
    // Create and setup registry
    let registry_id = env.register_contract(None, crate::VaspRegistryContract);
    let registry_client = crate::VaspRegistryContractClient::new(&env, &registry_id);
    registry_client.initialize(&admin);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    let threshold = 1000i128;
    router_client.initialize(&admin, &token_address, &registry_id, &threshold);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let another_to = Address::generate(&env);
    
    // Register both as VASPs
    let from_endpoint = String::from_str(&env, "https://vasp-from.example.com");
    let to_endpoint = String::from_str(&env, "https://vasp-to.example.com");
    let another_endpoint = String::from_str(&env, "https://vasp-another.example.com");
    
    registry_client.add_vasp(&admin, &from, &from_endpoint);
    registry_client.add_vasp(&admin, &to, &to_endpoint);
    registry_client.add_vasp(&admin, &another_to, &another_endpoint);
    
    // Mint tokens
    token_client.mint(&from, &10000i128);
    
    let hash = BytesN::random(&env);
    
    // First transfer with hash should succeed
    let result1 = router_client.try_compliant_transfer(&from, &to, &5000i128, &Some(hash.clone()));
    assert!(result1.is_ok());
    
    // Second transfer with same hash should fail (even to different recipient)
    let result2 = router_client.try_compliant_transfer(&from, &another_to, &2000i128, &Some(hash));
    assert_eq!(result2, Err(Ok(RouterError::HashAlreadyUsed)));
}

#[test]
fn test_unregistered_sender_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, token_client) = create_token_contract(&env);
    
    // Create and setup registry
    let registry_id = env.register_contract(None, crate::VaspRegistryContract);
    let registry_client = crate::VaspRegistryContractClient::new(&env, &registry_id);
    registry_client.initialize(&admin);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    let threshold = 1000i128;
    router_client.initialize(&admin, &token_address, &registry_id, &threshold);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    // Only register 'to' as a VASP, leave 'from' unregistered
    let to_endpoint = String::from_str(&env, "https://vasp-to.example.com");
    registry_client.add_vasp(&admin, &to, &to_endpoint);
    
    token_client.mint(&from, &5000i128);
    
    let hash = BytesN::random(&env);
    
    // Should fail due to unregistered sender
    let result = router_client.try_compliant_transfer(&from, &to, &5000i128, &Some(hash));
    assert_eq!(result, Err(Ok(RouterError::UnregisteredCounterparty)));
}

#[test]
fn test_unregistered_recipient_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, token_client) = create_token_contract(&env);
    
    // Create and setup registry
    let registry_id = env.register_contract(None, crate::VaspRegistryContract);
    let registry_client = crate::VaspRegistryContractClient::new(&env, &registry_id);
    registry_client.initialize(&admin);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    let threshold = 1000i128;
    router_client.initialize(&admin, &token_address, &registry_id, &threshold);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    // Only register 'from' as a VASP, leave 'to' unregistered
    let from_endpoint = String::from_str(&env, "https://vasp-from.example.com");
    registry_client.add_vasp(&admin, &from, &from_endpoint);
    
    token_client.mint(&from, &5000i128);
    
    let hash = BytesN::random(&env);
    
    // Should fail due to unregistered recipient
    let result = router_client.try_compliant_transfer(&from, &to, &5000i128, &Some(hash));
    assert_eq!(result, Err(Ok(RouterError::UnregisteredCounterparty)));
}

#[test]
fn test_admin_can_update_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env);
    let registry = Address::generate(&env);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    router_client.initialize(&admin, &token_address, &registry, &1000i128);
    
    assert_eq!(router_client.get_threshold(), 1000i128);
    
    router_client.set_threshold(&admin, &5000i128);
    
    assert_eq!(router_client.get_threshold(), 5000i128);
}

#[test]
fn test_non_admin_cannot_update_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let (token_address, _) = create_token_contract(&env);
    let registry = Address::generate(&env);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    router_client.initialize(&admin, &token_address, &registry, &1000i128);
    
    // Unauthorized user should panic
    let result = std::panic::catch_unwind(|| {
        router_client.set_threshold(&unauthorized, &5000i128);
    });
    assert!(result.is_err());
}

#[test]
fn test_successful_compliant_transfer_updates_balances() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, token_client) = create_token_contract(&env);
    
    // Create and setup registry
    let registry_id = env.register_contract(None, crate::VaspRegistryContract);
    let registry_client = crate::VaspRegistryContractClient::new(&env, &registry_id);
    registry_client.initialize(&admin);
    
    let router_id = env.register_contract(None, RouterContract);
    let router_client = RouterContractClient::new(&env, &router_id);
    
    let threshold = 1000i128;
    router_client.initialize(&admin, &token_address, &registry_id, &threshold);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    // Register both as VASPs
    let from_endpoint = String::from_str(&env, "https://vasp-from.example.com");
    let to_endpoint = String::from_str(&env, "https://vasp-to.example.com");
    
    registry_client.add_vasp(&admin, &from, &from_endpoint);
    registry_client.add_vasp(&admin, &to, &to_endpoint);
    
    // Mint tokens to sender
    token_client.mint(&from, &10000i128);
    
    let hash = BytesN::random(&env);
    
    // Execute compliant transfer
    let result = router_client.try_compliant_transfer(&from, &to, &5000i128, &Some(hash));
    assert!(result.is_ok());
    
    // Verify balances updated
    assert_eq!(token_client.balance(&from), 5000i128);
    assert_eq!(token_client.balance(&to), 5000i128);
}
