#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Verify initialization worked by trying to add a VASP
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    env.mock_all_auths();
    client.add_vasp(&admin, &vasp, &endpoint);

    assert!(client.is_registered(&vasp));
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin); // Should panic
}

#[test]
fn test_only_admin_can_add_vasp() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    client.initialize(&admin);

    env.mock_all_auths();
    
    // Unauthorized user should panic
    let result = std::panic::catch_unwind(|| {
        client.add_vasp(&unauthorized, &vasp, &endpoint);
    });
    assert!(result.is_err());
}

#[test]
fn test_add_and_query_vasp() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    client.initialize(&admin);

    env.mock_all_auths();
    client.add_vasp(&admin, &vasp, &endpoint);

    assert!(client.is_registered(&vasp));
    assert_eq!(client.get_endpoint(&vasp), Some(endpoint));
}

#[test]
fn test_is_registered_returns_false_for_unknown() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unknown_vasp = Address::generate(&env);

    client.initialize(&admin);

    assert!(!client.is_registered(&unknown_vasp));
    assert_eq!(client.get_endpoint(&unknown_vasp), None);
}

#[test]
fn test_remove_vasp_soft_deletes() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    client.initialize(&admin);

    env.mock_all_auths();
    client.add_vasp(&admin, &vasp, &endpoint);
    assert!(client.is_registered(&vasp));

    client.remove_vasp(&admin, &vasp);
    assert!(!client.is_registered(&vasp));
    assert_eq!(client.get_endpoint(&vasp), None);
}

#[test]
fn test_readd_removed_vasp_reactivates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");
    let new_endpoint = String::from_str(&env, "https://vasp-a-new.example.com");

    client.initialize(&admin);

    env.mock_all_auths();
    
    // Add, remove, then re-add
    client.add_vasp(&admin, &vasp, &endpoint);
    client.remove_vasp(&admin, &vasp);
    assert!(!client.is_registered(&vasp));

    client.add_vasp(&admin, &vasp, &new_endpoint);
    assert!(client.is_registered(&vasp));
    assert_eq!(client.get_endpoint(&vasp), Some(new_endpoint));
}

#[test]
fn test_only_admin_can_remove_vasp() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    client.initialize(&admin);

    env.mock_all_auths();
    client.add_vasp(&admin, &vasp, &endpoint);

    // Unauthorized user should panic
    let result = std::panic::catch_unwind(|| {
        client.remove_vasp(&unauthorized, &vasp);
    });
    assert!(result.is_err());
}

#[test]
fn test_transfer_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    client.initialize(&admin);

    env.mock_all_auths();
    client.transfer_admin(&admin, &new_admin);

    // Old admin should no longer be able to add VASPs
    let result = std::panic::catch_unwind(|| {
        client.add_vasp(&admin, &vasp, &endpoint);
    });
    assert!(result.is_err());

    // New admin should be able to add VASPs
    client.add_vasp(&new_admin, &vasp, &endpoint);
    assert!(client.is_registered(&vasp));
}

#[test]
fn test_only_admin_can_transfer_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.initialize(&admin);

    env.mock_all_auths();

    // Unauthorized user should panic
    let result = std::panic::catch_unwind(|| {
        client.transfer_admin(&unauthorized, &new_admin);
    });
    assert!(result.is_err());
}
