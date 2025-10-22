use alloy_primitives::{keccak256 as alloy_keccak256, B256};
use alloy_sol_types::{SolType, SolValue};
use cosmwasm_std::{Binary, Uint256};
use cw_wavs_sdk::contract_kinds::mirror::{
    MirrorStakeRegistryExecutor, MirrorStakeRegistryQuerier,
};
use k256::ecdsa::{
    signature::hazmat::PrehashSigner, RecoveryId, Signature, SigningKey, VerifyingKey,
};
use layer_climb_address::AddrEvm;
use rand::thread_rng;

fn create_eip191_hash(message: &[u8]) -> B256 {
    let prefix = b"\x19Ethereum Signed Message:\n";
    let message_len = message.len().to_string();

    let mut full_message = Vec::new();
    full_message.extend_from_slice(prefix);
    full_message.extend_from_slice(message_len.as_bytes());
    full_message.extend_from_slice(message);

    alloy_keccak256(&full_message)
}

fn create_signing_key_and_address() -> (SigningKey, AddrEvm) {
    let signing_key = SigningKey::random(&mut thread_rng());
    let eth_address = derive_eth_address_from_signing_key(&signing_key);
    (signing_key, eth_address)
}

fn derive_eth_address_from_signing_key(signing_key: &SigningKey) -> AddrEvm {
    let public_key = signing_key.verifying_key();
    let public_key_point = public_key.to_encoded_point(false);
    let public_key_bytes = &public_key_point.as_bytes()[1..]; // Skip 0x04 prefix

    let hash = alloy_keccak256(public_key_bytes);

    // Take last 20 bytes as Ethereum address
    let mut addr_bytes = [0u8; 20];
    addr_bytes.copy_from_slice(&hash[12..32]);
    AddrEvm::new(addr_bytes)
}

fn sign_message_hash(signing_key: &SigningKey, message_hash: &[u8]) -> Vec<u8> {
    let signature: Signature = signing_key.sign_prehash(message_hash).unwrap();
    let (r, s) = signature.split_bytes();

    let mut signature_bytes = Vec::with_capacity(65);
    signature_bytes.extend_from_slice(&r);
    signature_bytes.extend_from_slice(&s);

    // Find the correct recovery ID
    for recovery_id in 0..4u8 {
        signature_bytes.truncate(64);
        signature_bytes.push(recovery_id);

        // Test if this recovery ID works with k256 (same as contract will use)
        if let Ok(hash) = B256::try_from(message_hash) {
            // Extract r, s from signature_bytes
            let r_bytes: [u8; 32] = signature_bytes[0..32].try_into().unwrap();
            let s_bytes: [u8; 32] = signature_bytes[32..64].try_into().unwrap();

            if let Ok(k256_sig) = k256::ecdsa::Signature::from_scalars(r_bytes, s_bytes) {
                if let Ok(recovery_id) = RecoveryId::try_from(recovery_id) {
                    if let Ok(verifying_key) =
                        VerifyingKey::recover_from_prehash(hash.as_slice(), &k256_sig, recovery_id)
                    {
                        // Convert verifying key to Ethereum address
                        let public_key_bytes = verifying_key.to_encoded_point(false);
                        let public_key_uncompressed = &public_key_bytes.as_bytes()[1..]; // Skip 0x04 prefix
                        let addr_hash = alloy_keccak256(public_key_uncompressed);
                        let recovered_address = &addr_hash[12..]; // Last 20 bytes

                        let expected_addr = derive_eth_address_from_signing_key(signing_key);
                        if recovered_address == expected_addr.as_bytes() {
                            return signature_bytes;
                        }
                    }
                }
            }
        }
    }

    // If no valid recovery ID found, this is a cryptographic failure
    panic!("Failed to find valid recovery ID for signature - this indicates a cryptographic error")
}

pub async fn run_mirror_sanity_tests(
    executor: &MirrorStakeRegistryExecutor,
    querier: &MirrorStakeRegistryQuerier,
) {
    // Test: Basic operator registration and querying
    let operator = AddrEvm::new([0xaa; 20]);
    let signing_key = AddrEvm::new([0xbb; 20]);
    let weight = Uint256::from(1000u128);

    executor
        .set_operator_details(operator.clone(), signing_key.clone(), weight)
        .await
        .unwrap();

    // Verify operator weight
    let queried_weight = querier.get_operator_weight(operator.clone()).await.unwrap();
    assert_eq!(queried_weight, weight, "Operator weight should match");

    // Verify signing key
    let queried_key = querier.get_operator_signing_key(operator).await.unwrap();
    assert_eq!(queried_key, Some(signing_key), "Signing key should match");

    // Verify total weight
    let total_weight = querier.get_total_weight().await.unwrap();
    assert!(
        total_weight >= weight,
        "Total weight should include operator"
    );

    // Test: Batch operator management
    let batch_operators = vec![AddrEvm::new([0xdd; 20]), AddrEvm::new([0xee; 20])];
    let batch_keys = vec![AddrEvm::new([0xff; 20]), AddrEvm::new([0x00; 20])];
    let batch_weights = vec![Uint256::from(300u128), Uint256::from(200u128)];

    executor
        .batch_set_operator_details(
            batch_operators.clone(),
            batch_keys.clone(),
            batch_weights.clone(),
        )
        .await
        .unwrap();

    // Verify batch operations worked
    for (i, operator) in batch_operators.iter().enumerate() {
        let weight = querier.get_operator_weight(operator.clone()).await.unwrap();
        assert_eq!(
            weight, batch_weights[i],
            "Batch operator weight should match"
        );
    }

    // Test: Real signature validation with cryptography
    // Use a much higher weight to ensure signature validation passes even with state pollution
    let (test_signing_key, test_signing_addr) = create_signing_key_and_address();
    let test_operator = AddrEvm::new([0x11; 20]);
    let test_weight = Uint256::from(5000u128); // Higher weight to dominate any existing operators

    executor
        .set_operator_details(test_operator, test_signing_addr.clone(), test_weight)
        .await
        .unwrap();

    // Create and sign a test message
    let test_message = b"sanity signature test";
    let digest = create_eip191_hash(test_message);
    let signature = sign_message_hash(&test_signing_key, digest.as_slice());

    // Create ABI-encoded signature data
    use alloy_sol_types::sol_data::*;
    type SignatureDataType = (Array<Address>, Array<Bytes>, Uint<32>);

    let signers = vec![alloy_primitives::Address::from_slice(
        &test_signing_addr.as_bytes(),
    )];
    let signatures = vec![alloy_primitives::Bytes::copy_from_slice(&signature)];
    let tuple_data = (signers, signatures, 12345u32);
    let encoded_data = SignatureDataType::abi_encode(&tuple_data);

    // Test signature validation
    let result = querier
        .validate_signature(Binary::from(digest.to_vec()), Binary::from(encoded_data))
        .await
        .unwrap();

    assert!(result.is_valid, "Signature should be valid");
    assert_eq!(
        result.reference_block, 12345,
        "Reference block should match"
    );
    assert!(
        result.voting_power_signed > Uint256::zero(),
        "Should have voting power"
    );

    tracing::info!("Mirror sanity tests completed successfully");
}

pub async fn run_mirror_abi_signature_validation_test(
    executor: &MirrorStakeRegistryExecutor,
    querier: &MirrorStakeRegistryQuerier,
) {
    // Create real signing keys and addresses
    let (signing_key1, signing_address1) = create_signing_key_and_address();
    let (signing_key2, signing_address2) = create_signing_key_and_address();

    // Use unique operator addresses to avoid conflicts with sanity test
    let operator1 = AddrEvm::new([0x33; 20]);
    let operator2 = AddrEvm::new([0x44; 20]);

    executor
        .set_operator_details(
            operator1.clone(),
            signing_address1.clone(),
            Uint256::from(600u128),
        )
        .await
        .unwrap();

    executor
        .set_operator_details(
            operator2.clone(),
            signing_address2.clone(),
            Uint256::from(500u128),
        )
        .await
        .unwrap();

    // Create a deterministic test message hash using EIP-191 format
    let test_message = b"test message for signature validation";
    let digest = create_eip191_hash(test_message);

    // Create proper ECDSA signatures
    let sig1 = sign_message_hash(&signing_key1, digest.as_slice());
    let sig2 = sign_message_hash(&signing_key2, digest.as_slice());

    // Create ABI-encoded signature data
    let signers = vec![
        alloy_primitives::Address::from_slice(&signing_address1.as_bytes()),
        alloy_primitives::Address::from_slice(&signing_address2.as_bytes()),
    ];

    let signatures = vec![
        alloy_primitives::Bytes::copy_from_slice(&sig1),
        alloy_primitives::Bytes::copy_from_slice(&sig2),
    ];
    let tuple_data = (signers, signatures, 12345u32);
    let encoded_data = (tuple_data).abi_encode();

    // Test signature validation
    let digest_binary = Binary::from(digest.to_vec());
    let signature_data = Binary::from(encoded_data);

    let result = querier
        .validate_signature(digest_binary, signature_data)
        .await;

    let validation_result = result.unwrap();

    // Verify the validation results
    assert_eq!(
        validation_result.total_voting_power,
        Uint256::from(1100u128)
    );
    assert_eq!(validation_result.reference_block, 12345);
    assert_eq!(
        validation_result.voting_power_signed,
        Uint256::from(1100u128)
    );
    assert!(validation_result.is_valid);
}

pub async fn run_mirror_abi_binary_compatibility_test() {
    // Test that we can round-trip ABI encoding/decoding exactly like Solidity
    let original_operators = [[0x11; 20], [0x22; 20], [0x33; 20]];

    let original_signatures = [vec![0x01; 65], vec![0x02; 65], vec![0x03; 65]];

    let original_block = 99999u32;

    // Encode using alloy-sol-types (same as Solidity)
    let signers: Vec<alloy_primitives::Address> = original_operators
        .iter()
        .map(|op| alloy_primitives::Address::from_slice(op))
        .collect();

    let signatures: Vec<alloy_primitives::Bytes> = original_signatures
        .iter()
        .map(|sig| alloy_primitives::Bytes::copy_from_slice(sig))
        .collect();

    let tuple_data = (signers.clone(), signatures.clone(), original_block);
    let encoded = (tuple_data).abi_encode();

    // Decode it back
    type SignatureDataType = (
        alloy_sol_types::sol_data::Array<alloy_sol_types::sol_data::Address>,
        alloy_sol_types::sol_data::Array<alloy_sol_types::sol_data::Bytes>,
        alloy_sol_types::sol_data::Uint<32>,
    );
    let (decoded_addrs, decoded_sigs, decoded_block): (
        Vec<alloy_primitives::Address>,
        Vec<alloy_primitives::Bytes>,
        u32,
    ) = SignatureDataType::abi_decode(&encoded).unwrap();

    // Verify round-trip works
    assert_eq!(decoded_addrs.len(), 3);
    assert_eq!(decoded_sigs.len(), 3);
    assert_eq!(decoded_block, original_block);

    // Check operators
    for (i, addr) in decoded_addrs.iter().enumerate() {
        assert_eq!(addr.as_slice(), &original_operators[i]);
    }

    // Check signatures
    for (i, sig) in decoded_sigs.iter().enumerate() {
        assert_eq!(sig.as_ref(), &original_signatures[i]);
    }
}

pub async fn run_mirror_negative_test_scenarios(
    executor: &MirrorStakeRegistryExecutor,
    querier: &MirrorStakeRegistryQuerier,
) {
    // Test: Invalid signature validation with malformed ABI data
    let test_message = b"test invalid signature";
    let digest = create_eip191_hash(test_message);
    let digest_binary = Binary::from(digest.to_vec());

    // Malformed signature data (empty)
    let malformed_data = Binary::from(vec![]);
    let result = querier
        .validate_signature(digest_binary.clone(), malformed_data)
        .await;
    assert!(result.is_err(), "Should fail with malformed signature data");

    // Test: Zero weight operator handling
    let zero_weight_operator = AddrEvm::new([0x99; 20]);
    let zero_signing_key = AddrEvm::new([0x88; 20]);

    let result = executor
        .set_operator_details(
            zero_weight_operator.clone(),
            zero_signing_key.clone(),
            Uint256::zero(),
        )
        .await;

    if result.is_ok() {
        let weight = querier
            .get_operator_weight(zero_weight_operator)
            .await
            .unwrap();
        assert_eq!(weight, Uint256::zero());
    }

    // Test: Duplicate operator registration (should update, not error)
    let duplicate_operator = AddrEvm::new([0x77; 20]);
    let first_key = AddrEvm::new([0x66; 20]);
    let second_key = AddrEvm::new([0x55; 20]);
    let first_weight = Uint256::from(100u128);
    let second_weight = Uint256::from(200u128);

    // First registration
    executor
        .set_operator_details(duplicate_operator.clone(), first_key.clone(), first_weight)
        .await
        .unwrap();

    // Second registration (should update)
    executor
        .set_operator_details(
            duplicate_operator.clone(),
            second_key.clone(),
            second_weight,
        )
        .await
        .unwrap();

    // Should have updated values, not first values
    let final_weight = querier
        .get_operator_weight(duplicate_operator.clone())
        .await
        .unwrap();
    let final_key = querier
        .get_operator_signing_key(duplicate_operator)
        .await
        .unwrap();

    assert_eq!(final_weight, second_weight, "Weight should be updated");
    assert_eq!(final_key, Some(second_key), "Signing key should be updated");

    // Test: Invalid signature with correct ABI format but wrong signature
    let (valid_key, valid_addr) = create_signing_key_and_address();
    let invalid_operator = AddrEvm::new([0x45; 20]);

    executor
        .set_operator_details(invalid_operator, valid_addr.clone(), Uint256::from(300u128))
        .await
        .unwrap();

    // Create signature with different message but use it for original message
    let wrong_message = b"wrong message";
    let wrong_digest = create_eip191_hash(wrong_message);
    let wrong_sig = sign_message_hash(&valid_key, wrong_digest.as_slice());

    let signers = vec![alloy_primitives::Address::from_slice(
        &valid_addr.as_bytes(),
    )];
    let signatures = vec![alloy_primitives::Bytes::copy_from_slice(&wrong_sig)];
    let tuple_data = (signers, signatures, 54321u32);
    let encoded_data = (tuple_data).abi_encode();

    let original_message = b"original message";
    let original_digest = create_eip191_hash(original_message);
    let result = querier
        .validate_signature(
            Binary::from(original_digest.to_vec()),
            Binary::from(encoded_data),
        )
        .await;

    // This should either fail or return is_valid: false
    if let Ok(validation_result) = result {
        assert!(!validation_result.is_valid, "Should be invalid signature");
    }
}
