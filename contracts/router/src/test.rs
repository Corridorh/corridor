#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, BytesN, Env, String,
};
use vasp_registry::{VaspRegistryContract, VaspRegistryContractClient};

/// Create a Stellar Asset Contract and return its address plus the SEP-41 and
/// admin clients.
fn create_token<'a>(
    env: &Env,
    admin: &Address,
) -> (Address, TokenClient<'a>, StellarAssetClient<'a>) {
    let address = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    (
        address.clone(),
        TokenClient::new(env, &address),
        StellarAssetClient::new(env, &address),
    )
}

/// Deterministic 32-byte value used as a stand-in for a metadata commitment.
fn hash_in(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// Initialize a router + registry pair with two registered VASPs.
struct Fixture<'a> {
    admin: Address,
    token: TokenClient<'a>,
    sac: StellarAssetClient<'a>,
    router: RouterContractClient<'a>,
    registry: VaspRegistryContractClient<'a>,
    from: Address,
    to: Address,
}

fn setup<'a>(env: &'a Env, threshold: i128) -> Fixture<'a> {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let (token_address, token, sac) = create_token(env, &admin);

    let registry_id = env.register_contract(None, VaspRegistryContract);
    let registry = VaspRegistryContractClient::new(env, &registry_id);
    registry.initialize(&admin);

    let router_id = env.register_contract(None, RouterContract);
    let router = RouterContractClient::new(env, &router_id);
    router.initialize(&admin, &token_address, &registry_id, &threshold);

    let from = Address::generate(env);
    let to = Address::generate(env);

    Fixture {
        admin,
        token,
        sac,
        router,
        registry,
        from,
        to,
    }
}

fn assert_router_err<R: core::fmt::Debug, E: core::fmt::Debug>(
    result: Result<R, Result<RouterError, E>>,
    expected: RouterError,
) {
    match result {
        Err(Ok(err)) => assert_eq!(err, expected),
        other => panic!("expected contract error {expected:?}, got {other:?}"),
    }
}

#[test]
fn test_initialize_and_getters() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, _, _) = create_token(&env, &admin);
    let registry = Address::generate(&env);

    let router_id = env.register_contract(None, RouterContract);
    let router = RouterContractClient::new(&env, &router_id);
    router.initialize(&admin, &token_address, &registry, &1000);

    assert_eq!(router.get_threshold(), 1000);
    assert_eq!(router.get_admin(), admin);
    assert_eq!(router.get_token(), token_address);
    assert_eq!(router.get_registry(), registry);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, _, _) = create_token(&env, &admin);
    let registry = Address::generate(&env);

    let router_id = env.register_contract(None, RouterContract);
    let router = RouterContractClient::new(&env, &router_id);

    router.initialize(&admin, &token_address, &registry, &1000);
    router.initialize(&admin, &token_address, &registry, &1000); // should panic
}

#[test]
#[should_panic(expected = "Threshold must be non-negative")]
fn test_initialize_rejects_negative_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let (token_address, _, _) = create_token(&env, &admin);
    let registry = Address::generate(&env);

    let router_id = env.register_contract(None, RouterContract);
    let router = RouterContractClient::new(&env, &router_id);
    router.initialize(&admin, &token_address, &registry, &-1);
}

#[test]
fn test_transfer_below_threshold_succeeds_without_metadata() {
    let env = Env::default();
    let f = setup(&env, 1000);
    f.sac.mint(&f.from, &500);

    assert!(f
        .router
        .try_compliant_transfer(&f.from, &f.to, &500, &None)
        .is_ok());
    assert_eq!(f.token.balance(&f.to), 500);
}

#[test]
fn test_transfer_at_threshold_requires_metadata() {
    let env = Env::default();
    let f = setup(&env, 1000);

    let result = f
        .router
        .try_compliant_transfer(&f.from, &f.to, &1000, &None);
    assert_router_err(result, RouterError::MissingMetadata);
}

#[test]
fn test_transfer_above_threshold_requires_metadata() {
    let env = Env::default();
    let f = setup(&env, 1000);

    let result = f
        .router
        .try_compliant_transfer(&f.from, &f.to, &5000, &None);
    assert_router_err(result, RouterError::MissingMetadata);
}

#[test]
fn test_compliant_transfer_rejects_non_positive_amount() {
    let env = Env::default();
    let f = setup(&env, 1000);

    assert_router_err(
        f.router.try_compliant_transfer(&f.from, &f.to, &0, &None),
        RouterError::InvalidAmount,
    );
    assert_router_err(
        f.router.try_compliant_transfer(&f.from, &f.to, &-5, &None),
        RouterError::InvalidAmount,
    );
}

#[test]
fn test_successful_compliant_transfer_updates_balances_and_consumes_hash() {
    let env = Env::default();
    let f = setup(&env, 1000);

    let from_endpoint = String::from_str(&env, "https://vasp-from.example.com");
    let to_endpoint = String::from_str(&env, "https://vasp-to.example.com");
    f.registry.add_vasp(&f.admin, &f.from, &from_endpoint);
    f.registry.add_vasp(&f.admin, &f.to, &to_endpoint);

    f.sac.mint(&f.from, &10_000);
    let h = hash_in(&env, 7);

    assert!(f
        .router
        .try_compliant_transfer(&f.from, &f.to, &5000, &Some(h.clone()))
        .is_ok());

    assert_eq!(f.token.balance(&f.from), 5000);
    assert_eq!(f.token.balance(&f.to), 5000);
    assert!(f.router.is_hash_used(&h));
}

#[test]
fn test_hash_replay_protection() {
    let env = Env::default();
    let f = setup(&env, 1000);

    let endpoint = String::from_str(&env, "https://vasp.example.com");
    f.registry.add_vasp(&f.admin, &f.from, &endpoint);
    f.registry.add_vasp(&f.admin, &f.to, &endpoint);
    let another = Address::generate(&env);
    f.registry.add_vasp(&f.admin, &another, &endpoint);

    f.sac.mint(&f.from, &10_000);
    let h = hash_in(&env, 11);

    assert!(f
        .router
        .try_compliant_transfer(&f.from, &f.to, &5000, &Some(h.clone()))
        .is_ok());

    // Same hash, even to a different recipient, must be rejected.
    let result = f
        .router
        .try_compliant_transfer(&f.from, &another, &2000, &Some(h));
    assert_router_err(result, RouterError::HashAlreadyUsed);
}

#[test]
fn test_unregistered_sender_fails() {
    let env = Env::default();
    let f = setup(&env, 1000);

    // Only `to` is registered.
    let endpoint = String::from_str(&env, "https://vasp-to.example.com");
    f.registry.add_vasp(&f.admin, &f.to, &endpoint);
    f.sac.mint(&f.from, &5000);

    let result = f
        .router
        .try_compliant_transfer(&f.from, &f.to, &5000, &Some(hash_in(&env, 1)));
    assert_router_err(result, RouterError::UnregisteredCounterparty);
}

#[test]
fn test_unregistered_recipient_fails() {
    let env = Env::default();
    let f = setup(&env, 1000);

    // Only `from` is registered.
    let endpoint = String::from_str(&env, "https://vasp-from.example.com");
    f.registry.add_vasp(&f.admin, &f.from, &endpoint);
    f.sac.mint(&f.from, &5000);

    let result = f
        .router
        .try_compliant_transfer(&f.from, &f.to, &5000, &Some(hash_in(&env, 2)));
    assert_router_err(result, RouterError::UnregisteredCounterparty);
}

#[test]
fn test_removed_vasp_cannot_transfer() {
    let env = Env::default();
    let f = setup(&env, 1000);

    let endpoint = String::from_str(&env, "https://vasp.example.com");
    f.registry.add_vasp(&f.admin, &f.from, &endpoint);
    f.registry.add_vasp(&f.admin, &f.to, &endpoint);
    f.registry.remove_vasp(&f.admin, &f.to);

    f.sac.mint(&f.from, &5000);

    let result = f
        .router
        .try_compliant_transfer(&f.from, &f.to, &5000, &Some(hash_in(&env, 3)));
    assert_router_err(result, RouterError::UnregisteredCounterparty);
}

#[test]
fn test_admin_can_update_threshold() {
    let env = Env::default();
    let f = setup(&env, 1000);

    assert_eq!(f.router.get_threshold(), 1000);
    assert!(f.router.try_set_threshold(&f.admin, &5000).is_ok());
    assert_eq!(f.router.get_threshold(), 5000);
}

#[test]
fn test_non_admin_cannot_update_threshold() {
    let env = Env::default();
    let f = setup(&env, 1000);

    let unauthorized = Address::generate(&env);
    let result = f.router.try_set_threshold(&unauthorized, &5000);
    assert_router_err(result, RouterError::Unauthorized);
    assert_eq!(f.router.get_threshold(), 1000);
}

#[test]
fn test_set_threshold_rejects_negative() {
    let env = Env::default();
    let f = setup(&env, 1000);

    let result = f.router.try_set_threshold(&f.admin, &-1);
    assert_router_err(result, RouterError::InvalidThreshold);
}

#[test]
fn test_hash_is_not_used_by_default() {
    let env = Env::default();
    let f = setup(&env, 1000);
    assert!(!f.router.is_hash_used(&hash_in(&env, 42)));
}
