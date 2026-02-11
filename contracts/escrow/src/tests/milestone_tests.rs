#![cfg(test)]

extern crate std;

use crate::storage::types::{Escrow, Flags, Milestone, MilestoneUpdate, Roles, Trustline};

use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use super::helpers::{create_escrow_contract, create_usdc_token};

#[test]
fn test_change_milestone_status_and_approved_flag() {
    let env = Env::default();
    env.mock_all_auths();

    let approver_address = Address::generate(&env);
    let service_provider_address = Address::generate(&env);
    let admin = Address::generate(&env);
    let platform_address = Address::generate(&env);
    let release_signer_address = Address::generate(&env);
    let dispute_resolver_address = Address::generate(&env);
    let platform_fee = 3 * 100;

    let usdc_token = create_usdc_token(&env, &admin);

    let engagement_id = String::from_str(&env, "41431");

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

    let escrow_properties: Escrow = Escrow {
        engagement_id: engagement_id.clone(),
        title: String::from_str(&env, "Test Escrow"),
        description: String::from_str(&env, "Test Escrow Description"),
        roles: roles.clone(),
        platform_fee: platform_fee,
        milestones: milestones,
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let escrow_approver = test_data.client;

    escrow_approver.initialize_escrow(&escrow_properties);

    // Change milestone status (valid case)
    let milestone_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    escrow_approver.change_milestone_status(
        &milestone_updates,
        &service_provider_address,
    );

    // Verify milestone status change
    let updated_escrow = escrow_approver.get_escrow();
    assert_eq!(updated_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "completed"));

    // Change milestone approved_flag (valid case)
    let milestone_indexes = vec![&env, 0];
    escrow_approver.approve_milestones(&milestone_indexes, &approver_address);

    // Verify milestone approved_flag change
    let final_escrow = escrow_approver.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().flags.approved);

    // Invalid index test
    let invalid_index = 10;

    // Test for `change_status` with invalid index
    let invalid_updates = vec![
        &env,
        MilestoneUpdate {
            index: invalid_index,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &invalid_updates,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test for `change_approved_flag` with invalid index
    let invalid_indices = vec![&env, invalid_index];
    let result = escrow_approver.try_approve_milestones(&invalid_indices, &approver_address);
    assert!(result.is_err());

    // Test only authorized party can perform the function
    let unauthorized_address = Address::generate(&env);

    // Test for `change_status` by invalid service provider
    let valid_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "New evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &valid_updates,
        &unauthorized_address,
    );
    assert!(result.is_err());

    // Test for `change_approved_flag` by invalid approver
    let valid_indices = vec![&env, 0];
    let result = escrow_approver.try_approve_milestones(&valid_indices, &unauthorized_address);
    assert!(result.is_err());

    // Test changing multiple milestones at once
    let multiple_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
        MilestoneUpdate {
            index: 1,
            status: String::from_str(&env, "reviewed"),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    
    escrow_approver.change_milestone_status(
        &multiple_updates,
        &service_provider_address,
    );

    let batch_updated_escrow = escrow_approver.get_escrow();
    assert_eq!(batch_updated_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "reviewed"));
    assert_eq!(batch_updated_escrow.milestones.get(1).unwrap().status, String::from_str(&env, "reviewed"));
    assert_eq!(
        batch_updated_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );
    assert_eq!(
        batch_updated_escrow.milestones.get(1).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );

    // Test with empty status
    let empty_status_update = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, ""),
            evidence: Some(String::from_str(&env, "Batch update evidence")),
        },
    ];
    let result = escrow_approver.try_change_milestone_status(
        &empty_status_update,
        &service_provider_address,
    );
    assert!(result.is_err());

    // Test with different status and evidence for each milestone
    let different_updates = vec![
        &env,
        MilestoneUpdate {
            index: 0,
            status: String::from_str(&env, "completed"),
            evidence: Some(String::from_str(&env, "Evidence for milestone 0")),
        },
        MilestoneUpdate {
            index: 1,
            status: String::from_str(&env, "in-progress"),
            evidence: None,
        },
    ];
    
    escrow_approver.change_milestone_status(
        &different_updates,
        &service_provider_address,
    );

    let final_check_escrow = escrow_approver.get_escrow();
    assert_eq!(final_check_escrow.milestones.get(0).unwrap().status, String::from_str(&env, "completed"));
    assert_eq!(final_check_escrow.milestones.get(1).unwrap().status, String::from_str(&env, "in-progress"));
    assert_eq!(
        final_check_escrow.milestones.get(0).unwrap().evidence,
        String::from_str(&env, "Evidence for milestone 0")
    );
    assert_eq!(
        final_check_escrow.milestones.get(1).unwrap().evidence,
        String::from_str(&env, "Batch update evidence")
    );
}

#[test]
fn test_approve_multiple_milestones_at_once() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let approver = Address::generate(&env);
    let service_provider = Address::generate(&env);
    let platform = Address::generate(&env);
    let release_signer = Address::generate(&env);
    let dispute_resolver = Address::generate(&env);
    let (token_client, token_admin) = create_usdc_token(&env, &admin);

    let roles = Roles {
        approver: approver.clone(),
        service_provider: service_provider.clone(),
        platform_address: platform.clone(),
        release_signer: release_signer.clone(),
        dispute_resolver: dispute_resolver.clone(),
        observers: vec![&env],
    };

    let flags = Flags {
        disputed: false,
        released: false,
        resolved: false,
        approved: false,
    };

    let trustline = Trustline {
        address: token_client.address.clone(),
    };

    let milestones = vec![
        &env,
        Milestone {
            description: String::from_str(&env, "Milestone 1"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 1"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 2"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 2"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 3"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 3"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
        Milestone {
            description: String::from_str(&env, "Milestone 4"),
            status: String::from_str(&env, "Completed"),
            amount: 25_000,
            evidence: String::from_str(&env, "Evidence 4"),
            flags: flags.clone(),
            receiver: service_provider.clone(),
        },
    ];

    let escrow_properties = Escrow {
        engagement_id: String::from_str(&env, "test_multiple_approval"),
        title: String::from_str(&env, "Test Multiple Approval"),
        description: String::from_str(&env, "Testing approval of multiple milestones at once"),
        roles: roles.clone(),
        platform_fee: 300,
        milestones: milestones.clone(),
        trustline: trustline.clone(),
        receiver_memo: 0,
    };

    let test_data = create_escrow_contract(&env);
    let client = test_data.client;

    client.initialize_escrow(&escrow_properties);

    token_admin.mint(&client.address, &100_000);

    let initial_escrow = client.get_escrow();
    assert!(!initial_escrow.milestones.get(0).unwrap().flags.approved);
    assert!(!initial_escrow.milestones.get(1).unwrap().flags.approved);
    assert!(!initial_escrow.milestones.get(2).unwrap().flags.approved);
    assert!(!initial_escrow.milestones.get(3).unwrap().flags.approved);

    let indices_batch_1 = vec![&env, 0, 1];
    client.approve_milestones(&indices_batch_1, &approver);

    let after_first_batch = client.get_escrow();
    assert!(after_first_batch.milestones.get(0).unwrap().flags.approved, "Milestone 0 should be approved");
    assert!(after_first_batch.milestones.get(1).unwrap().flags.approved, "Milestone 1 should be approved");
    assert!(!after_first_batch.milestones.get(2).unwrap().flags.approved, "Milestone 2 should not be approved yet");
    assert!(!after_first_batch.milestones.get(3).unwrap().flags.approved, "Milestone 3 should not be approved yet");

    let indices_batch_2 = vec![&env, 2, 3];
    client.approve_milestones(&indices_batch_2, &approver);

    let after_second_batch = client.get_escrow();
    assert!(after_second_batch.milestones.get(0).unwrap().flags.approved);
    assert!(after_second_batch.milestones.get(1).unwrap().flags.approved);
    assert!(after_second_batch.milestones.get(2).unwrap().flags.approved, "Milestone 2 should now be approved");
    assert!(after_second_batch.milestones.get(3).unwrap().flags.approved, "Milestone 3 should now be approved");

    let already_approved_indices = vec![&env, 0, 1];
    let result = client.try_approve_milestones(&already_approved_indices, &approver);
    assert!(result.is_err(), "Should fail when trying to approve already approved milestones");

    let invalid_indices = vec![&env, 10];
    let result = client.try_approve_milestones(&invalid_indices, &approver);
    assert!(result.is_err(), "Should fail with non-existent index");

    let test_data_2 = create_escrow_contract(&env);
    let client_2 = test_data_2.client;
    client_2.initialize_escrow(&escrow_properties);

    let mixed_indices = vec![&env, 0, 1, 99];
    let result = client_2.try_approve_milestones(&mixed_indices, &approver);
    assert!(result.is_err(), "Should fail when any index in the batch is invalid");

    let after_failed_attempt = client_2.get_escrow();
    assert!(!after_failed_attempt.milestones.get(0).unwrap().flags.approved, "Milestone 0 should not be approved after failed batch");
    assert!(!after_failed_attempt.milestones.get(1).unwrap().flags.approved, "Milestone 1 should not be approved after failed batch");

    let unauthorized = Address::generate(&env);
    let valid_indices = vec![&env, 0];
    let result = client_2.try_approve_milestones(&valid_indices, &unauthorized);
    assert!(result.is_err(), "Should fail when approver is not authorized");

    let all_indices = vec![&env, 0, 1, 2, 3];
    client_2.approve_milestones(&all_indices, &approver);

    let final_escrow = client_2.get_escrow();
    assert!(final_escrow.milestones.get(0).unwrap().flags.approved);
    assert!(final_escrow.milestones.get(1).unwrap().flags.approved);
    assert!(final_escrow.milestones.get(2).unwrap().flags.approved);
    assert!(final_escrow.milestones.get(3).unwrap().flags.approved);
}
