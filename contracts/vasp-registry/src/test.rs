#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (VaspRegistryContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

/// Assert that a `try_*` call failed with a specific contract error.
///
/// The generated `try_*` client methods return
/// `Result<Result<T, ConversionError>, Result<E, InvokeError>>`; a contract
/// error therefore surfaces as `Err(Ok(err))`.
fn assert_registry_err<R: core::fmt::Debug, E: core::fmt::Debug>(
    result: Result<R, Result<RegistryError, E>>,
    expected: RegistryError,
) {
    match result {
        Err(Ok(err)) => assert_eq!(err, expected),
        other => panic!("expected contract error {expected:?}, got {other:?}"),
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    assert_eq!(client.get_admin(), admin);

    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");
    client.add_vasp(&admin, &vasp, &endpoint);
    assert!(client.is_registered(&vasp));
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, VaspRegistryContract);
    let client = VaspRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin); // should panic
}

#[test]
fn test_only_admin_can_add_vasp() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let unauthorized = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    let result = client.try_add_vasp(&unauthorized, &vasp, &endpoint);
    assert_registry_err(result, RegistryError::Unauthorized);
    assert!(!client.is_registered(&vasp));
}

#[test]
fn test_add_and_query_vasp() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");
    client.add_vasp(&admin, &vasp, &endpoint);

    assert!(client.is_registered(&vasp));
    assert_eq!(client.get_endpoint(&vasp), Some(endpoint));
}

#[test]
fn test_is_registered_returns_false_for_unknown() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let unknown_vasp = Address::generate(&env);
    assert!(!client.is_registered(&unknown_vasp));
    assert_eq!(client.get_endpoint(&unknown_vasp), None);
}

#[test]
fn test_remove_vasp_soft_deletes() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    client.add_vasp(&admin, &vasp, &endpoint);
    assert!(client.is_registered(&vasp));

    client.remove_vasp(&admin, &vasp);
    assert!(!client.is_registered(&vasp));
    assert_eq!(client.get_endpoint(&vasp), None);
}

#[test]
fn test_remove_unknown_vasp_returns_not_found() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let vasp = Address::generate(&env);
    let result = client.try_remove_vasp(&admin, &vasp);
    assert_registry_err(result, RegistryError::NotFound);
}

#[test]
fn test_readd_removed_vasp_reactivates() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");
    let new_endpoint = String::from_str(&env, "https://vasp-a-new.example.com");

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
    let (client, admin) = setup(&env);

    let unauthorized = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");
    client.add_vasp(&admin, &vasp, &endpoint);

    let result = client.try_remove_vasp(&unauthorized, &vasp);
    assert_registry_err(result, RegistryError::Unauthorized);
    assert!(client.is_registered(&vasp));
}

#[test]
fn test_transfer_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let new_admin = Address::generate(&env);
    let vasp = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://vasp-a.example.com");

    client.transfer_admin(&admin, &new_admin);
    assert_eq!(client.get_admin(), new_admin);

    // Old admin should no longer be able to add VASPs.
    let result = client.try_add_vasp(&admin, &vasp, &endpoint);
    assert_registry_err(result, RegistryError::Unauthorized);

    // New admin should be able to add VASPs.
    client.add_vasp(&new_admin, &vasp, &endpoint);
    assert!(client.is_registered(&vasp));
}

#[test]
fn test_only_admin_can_transfer_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let unauthorized = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let result = client.try_transfer_admin(&unauthorized, &new_admin);
    assert_registry_err(result, RegistryError::Unauthorized);
    // Admin is unchanged.
    assert_eq!(client.get_admin(), admin);
}
