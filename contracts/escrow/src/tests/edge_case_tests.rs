#![cfg(test)]

extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, Map, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_distributions_size_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);
    let engagement_id = String::from_str(&env, "dist-limit-test");

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline: Trustline = Trustline {
        address: usdc_token.0.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Test milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount,
            evidence: String::from_str(&env, ""),
            receiver: service_provider_address.clone(),
        },
    ];

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Testing distributions limit"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        trustline,
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    usdc_token.1.mint(&admin, &amount);
    usdc_token.0.transfer(&admin, &client.address, &amount);

    client.dispute_milestone(&0, &approver_address);

    // Test 1: Try to resolve with more than 50 distributions (should fail)
    let mut large_dist = Map::new(&env);
    for _i in 0..51 {
        let addr = Address::generate(&env);
        large_dist.set(addr, 1_000_000);
    }
    
    let result = client.try_resolve_milestone_dispute(
        &dispute_resolver_address,
        &0,
        &trustless_work_address,
        &large_dist,
    );
    assert!(result.is_err(), "Should fail with more than 50 distributions");

    // Test 2: Try withdraw_remaining_funds with more than 50 distributions (should fail)
    let result = client.try_withdraw_remaining_funds(
        &dispute_resolver_address,
        &trustless_work_address,
        &large_dist,
    );
    assert!(result.is_err(), "Should fail with more than 50 distributions");

    // Test 3: Verify that exactly 50 distributions work
    let mut valid_dist = Map::new(&env);
    for _i in 0..50 {
        let addr = Address::generate(&env);
        valid_dist.set(addr, 1_000_000);
    }
    
    client.resolve_milestone_dispute(
        &dispute_resolver_address,
        &0,
        &trustless_work_address,
        &valid_dist,
    );

    let escrow = client.get_escrow();
    assert!(escrow.milestones.get(0).unwrap().flags.resolved);
}
