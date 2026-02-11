#![cfg(test)]

extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, Roles, Trustline};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_change_escrow_rejects_platform_fee_exceeding_aggregate_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);

    let usdc_token = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags { disputed: false, released: false, resolved: false, approved: false };

    let trustline: Trustline = Trustline { address: usdc_token.0.address.clone() };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "M1"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    // Start with a valid platform fee
    let base_platform_fee = 3 * 100;
    let escrow_properties: Escrow = Escrow {
        engagement_id: String::from_str(&env, "E1"),
        title: String::from_str(&env, "Escrow"),
        description: String::from_str(&env, "Desc"),
        roles: roles.clone(),
        platform_fee: base_platform_fee,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;
    client.initialize_escrow(&escrow_properties);

    // Now attempt to change to an invalid platform fee (aggregate > 100%). TW fee is 30 bps.
    let over_platform_fee = (10_000 - 30) + 1;
    let updated_escrow_properties = Escrow {
        platform_fee: over_platform_fee,
        ..escrow_properties
    };

    let res = client.try_update_escrow(&platform_address, &updated_escrow_properties);
    assert!(res.is_err());
}

#[test]
fn test_update_escrow() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let _receiver_address = Address::generate(&env);

    let amount: i128 = 100_000_000;
    let platform_fee = 3 * 100;
    let usdc_token = create_usdc_token(&env, &admin);

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

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let engagement_id = String::from_str(&env, "test_escrow_2");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Create a new updated escrow properties (no funds: can modify any field)
    let new_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            amount: amount * 2,
            flags: flags.clone(),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone updated"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            amount: amount * 2,
            flags: flags.clone(),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "Initial evidence"),
            amount: amount * 2,
            flags: flags.clone(),
            receiver: service_provider_address.clone(),
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow Updated"),
        description: String::from_str(&env, "Test Escrow Description Updated"),
        roles: roles.clone(),
        platform_fee: platform_fee * 2,
        milestones: new_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    // Update escrow properties
    let _updated_escrow =
        escrow_approver.update_escrow(&platform_address, &updated_escrow_properties);

    // Verify updated escrow properties
    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.title, updated_escrow_properties.title);
    assert_eq!(escrow.description, updated_escrow_properties.description);
    assert_eq!(escrow.platform_fee, updated_escrow_properties.platform_fee);
    assert_eq!(escrow.milestones, updated_escrow_properties.milestones);
    assert_eq!(
        escrow.roles.release_signer,
        updated_escrow_properties.roles.release_signer
    );
    assert_eq!(
        escrow.roles.dispute_resolver,
        updated_escrow_properties.roles.dispute_resolver
    );
    for (i, _) in escrow.milestones.iter().enumerate() {
        assert_eq!(
            escrow.milestones.get(i as u32).unwrap().receiver,
            updated_escrow_properties.milestones.get(i as u32).unwrap().receiver
        );
    }
    assert_eq!(
        escrow.receiver_memo,
        updated_escrow_properties.receiver_memo
    );

    // Try to update escrow properties without platform address (should fail)
    let non_platform_address = Address::generate(&env);
    let result =
        escrow_approver.try_update_escrow(&non_platform_address, &updated_escrow_properties);
    assert!(result.is_err());
}

#[test]
fn test_append_milestones_with_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;
    let amount: i128 = 100_000_000;

    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let roles: Roles = Roles {
        approver: approver_address.clone(),
        service_provider: service_provider_address.clone(),
        platform_address: platform_address.clone(),
        release_signer: release_signer_address.clone(),
        dispute_resolver: dispute_resolver_address.clone(),
        observers: vec![&env],
    };

    let flags: Flags = Flags { disputed: false, released: false, resolved: false, approved: false };

    let trustline: Trustline = Trustline { address: token_client.address.clone() };

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "First milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Second milestone"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 100_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let engagement_id = String::from_str(&env, "append_with_funds");
    let initial_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&initial_escrow_properties);

    // Fund the escrow (contract will hold funds)
    token_admin.mint(&release_signer_address, &amount);
    escrow_approver.fund_escrow(&release_signer_address, &initial_escrow_properties, &amount);

    // Now attempt to append new milestones while funds exist
    let updated_milestones = vec![
        &env,
        initial_escrow_properties.milestones.get(0).unwrap(),
        initial_escrow_properties.milestones.get(1).unwrap(),
        Milestone {
            description: String::from_str(&env, "Third milestone new"),
            status: String::from_str(&env, "Pending"),
            flags: flags.clone(),
            amount: 200_000,
            evidence: String::from_str(&env, "Empty"),
            receiver: service_provider_address.clone(),
        },
    ];

    let updated_escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: updated_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    escrow_approver.update_escrow(&platform_address, &updated_escrow_properties);

    let escrow = escrow_approver.get_escrow();
    assert_eq!(escrow.milestones.len(), 3);
    assert_eq!(escrow.milestones.get(0).unwrap(), initial_escrow_properties.milestones.get(0).unwrap());
    assert_eq!(escrow.milestones.get(1).unwrap(), initial_escrow_properties.milestones.get(1).unwrap());
    // Non-milestone fields must remain unchanged
    assert_eq!(escrow.engagement_id, initial_escrow_properties.engagement_id);
    assert_eq!(escrow.title, initial_escrow_properties.title);
    assert_eq!(escrow.description, initial_escrow_properties.description);
    assert!(escrow.roles == initial_escrow_properties.roles);
    assert_eq!(escrow.platform_fee, initial_escrow_properties.platform_fee);
    assert!(escrow.trustline == initial_escrow_properties.trustline);
    assert_eq!(escrow.receiver_memo, initial_escrow_properties.receiver_memo);
}

#[test]
fn test_update_after_milestone_approved_append_new() {
    let env = Env::default();
    env.mock_all_auths();

    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_client, _token_admin) = create_usdc_token(&env, &admin);

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        observers: vec![&env],
    };
    let flags = Flags { disputed: false, released: false, resolved: false, approved: false };
    let trustline = Trustline { address: token_client.address.clone() };

    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "m1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount: 100_000,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng-approved-update"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee: 300,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    let milestone_indexes = vec![&env, 0];
    client.approve_milestones(&milestone_indexes, &approver);
    let after_approval = client.get_escrow();
    let approved_milestone = after_approval.milestones.get(0).unwrap();
    assert!(approved_milestone.flags.approved, "Milestone should be approved before update");

    let new_milestone = Milestone {
        description: String::from_str(&env, "m2"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, "e"),
        amount: 150_000,
        flags: flags.clone(),
        receiver: service_provider.clone(),
    };
    let updated_milestones = vec![&env, approved_milestone.clone(), new_milestone.clone()];

    let updated_escrow = Escrow {
        engagement_id: esc.engagement_id.clone(),
        title: esc.title.clone(),
        description: esc.description.clone(),
        roles: esc.roles.clone(),
        platform_fee: esc.platform_fee,
        milestones: updated_milestones.clone(),
        trustline: esc.trustline.clone(),
        receiver_memo: esc.receiver_memo,
    };

    let res = client.try_update_escrow(&platform, &updated_escrow);
    assert!(res.is_ok(), "Update should succeed when appending new milestone with flags false while keeping existing approved milestone flags unchanged");

    let final_escrow = client.get_escrow();
    assert_eq!(final_escrow.milestones.len(), 2);
    assert!(final_escrow.milestones.get(0).unwrap().flags.approved, "Existing milestone approval flag must remain true");
    let appended = final_escrow.milestones.get(1).unwrap();
    assert!(!appended.flags.approved && !appended.flags.released && !appended.flags.resolved && !appended.flags.disputed, "New milestone flags must all be false");
}

#[test]
fn test_update_after_milestone_released_append_new() {
    let env = Env::default();
    env.mock_all_auths();

    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let trustless_work_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        observers: vec![&env],
    };
    let flags = Flags { disputed: false, released: false, resolved: false, approved: false };
    let trustline = Trustline { address: token_client.address.clone() };

    let amount: i128 = 100_000;
    let initial_milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "m1"),
            status: String::from_str(&env, "Pending"),
            evidence: String::from_str(&env, "e"),
            amount,
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let esc = Escrow {
        engagement_id: String::from_str(&env, "eng-released-update"),
        title: String::from_str(&env, "t"),
        description: String::from_str(&env, "d"),
        roles: roles.clone(),
        platform_fee: 300,
        milestones: initial_milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test = create_escrow_contract(&env);
    let client = test.client;
    client.initialize_escrow(&esc);

    token_admin.mint(&client.address, &amount);
    let milestone_indexes = vec![&env, 0];
    client.approve_milestones(&milestone_indexes, &approver);
    client.release_milestone_funds(&release_signer, &trustless_work_address, &0);

    let after_release = client.get_escrow();
    let released_milestone = after_release.milestones.get(0).unwrap();
    assert!(released_milestone.flags.released, "Milestone should be released before update");

    let new_milestone = Milestone {
        description: String::from_str(&env, "m2"),
        status: String::from_str(&env, "Pending"),
        evidence: String::from_str(&env, "e"),
        amount,
        flags: flags.clone(),
        receiver: service_provider.clone(),
    };
    let updated_milestones = vec![&env, released_milestone.clone(), new_milestone.clone()];

    let updated_escrow = Escrow {
        engagement_id: esc.engagement_id.clone(),
        title: esc.title.clone(),
        description: esc.description.clone(),
        roles: esc.roles.clone(),
        platform_fee: esc.platform_fee,
        milestones: updated_milestones.clone(),
        trustline: esc.trustline.clone(),
        receiver_memo: esc.receiver_memo,
    };

    let res = client.try_update_escrow(&platform, &updated_escrow);
    assert!(res.is_ok(), "Update should succeed when appending after a milestone was released");

    let final_escrow = client.get_escrow();
    assert_eq!(final_escrow.milestones.len(), 2);
    assert!(final_escrow.milestones.get(0).unwrap().flags.released, "Existing milestone released flag must remain true");
    let appended = final_escrow.milestones.get(1).unwrap();
    assert!(!appended.flags.approved && !appended.flags.released && !appended.flags.resolved && !appended.flags.disputed, "New milestone flags must all be false");
}
