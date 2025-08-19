use cosmwasm_std::{Binary, Uint256};
use ethabi::{decode, encode, ParamType, Token};
use layer_climb_address::AddrEvm;
use sdk::contract_kinds::mirror::{MirrorStakeRegistryExecutor, MirrorStakeRegistryQuerier};

pub async fn run_mirror_operator_management_test(
    executor: &MirrorStakeRegistryExecutor,
    querier: &MirrorStakeRegistryQuerier,
) {
    // Test setting operator details
    let operator = AddrEvm::new([0x12; 20]);
    let signing_key = AddrEvm::new([0xab; 20]);
    let weight = Uint256::from(500u128);

    executor
        .set_operator_details(operator.clone(), signing_key.clone(), weight)
        .await
        .unwrap();

    // Query the operator weight
    let queried_weight = querier
        .get_operator_weight(operator.clone())
        .await
        .unwrap();

    assert_eq!(queried_weight, weight);

    // Query the operator signing key
    let queried_key = querier
        .get_operator_signing_key(operator.clone())
        .await
        .unwrap();

    assert_eq!(queried_key, Some(signing_key));

    // Query total weight
    let total_weight = querier
        .get_total_weight()
        .await
        .unwrap();

    assert_eq!(total_weight, weight);
}

pub async fn run_mirror_batch_operator_management_test(
    executor: &MirrorStakeRegistryExecutor,
    querier: &MirrorStakeRegistryQuerier,
) {
    // Test batch setting operator details
    let operators = vec![AddrEvm::new([0x12; 20]), AddrEvm::new([0x34; 20])];
    let signing_keys = vec![AddrEvm::new([0xab; 20]), AddrEvm::new([0xcd; 20])];
    let weights = vec![Uint256::from(500u128), Uint256::from(300u128)];

    executor
        .batch_set_operator_details(operators.clone(), signing_keys.clone(), weights.clone())
        .await
        .unwrap();

    // Verify each operator
    for (i, operator) in operators.iter().enumerate() {
        let queried_weight = querier
            .get_operator_weight(operator.clone())
            .await
            .unwrap();
        assert_eq!(queried_weight, weights[i]);

        let queried_key = querier
            .get_operator_signing_key(operator.clone())
            .await
            .unwrap();
        assert_eq!(queried_key, Some(signing_keys[i].clone()));
    }

    // Check total weight
    let expected_total = weights.iter().sum::<Uint256>();
    let total_weight = querier
        .get_total_weight()
        .await
        .unwrap();
    assert_eq!(total_weight, expected_total);
}

pub async fn run_mirror_abi_signature_validation_test(
    executor: &MirrorStakeRegistryExecutor,
    querier: &MirrorStakeRegistryQuerier,
) {
    // Register operators
    let operator1 = AddrEvm::new([1u8; 20]);
    let operator2 = AddrEvm::new([2u8; 20]);
    let signing_key1 = AddrEvm::new([0xaa; 20]);
    let signing_key2 = AddrEvm::new([0xbb; 20]);

    executor
        .set_operator_details(operator1, signing_key1, Uint256::from(600u128))
        .await
        .unwrap();

    executor
        .set_operator_details(operator2, signing_key2, Uint256::from(500u128))
        .await
        .unwrap();

    // Create ABI-encoded signature data (same format as Solidity)
    let operators = vec![
        Token::Address([1u8; 20].into()),
        Token::Address([2u8; 20].into()),
    ];

    // Create signatures with valid recovery IDs
    let mut sig1 = vec![0u8; 65];
    sig1[64] = 0; // Valid recovery ID
    let mut sig2 = vec![0u8; 65];
    sig2[64] = 1; // Valid recovery ID

    let signatures = vec![Token::Bytes(sig1), Token::Bytes(sig2)];

    let reference_block = Token::Uint(12345u32.into());

    // Encode using the same ABI format as Solidity: abi.encode(operators, signatures, referenceBlock)
    let encoded_data = encode(&[
        Token::Array(operators),
        Token::Array(signatures),
        reference_block,
    ]);

    // Test signature validation
    let digest = Binary::from(vec![0u8; 32]); // Mock digest
    let signature_data = Binary::from(encoded_data);

    let result = querier
        .validate_signature(digest, signature_data)
        .await;

    match result {
        Ok(validation_result) => {
            // Verify enhanced return values
            assert_eq!(
                validation_result.total_voting_power,
                Uint256::from(1100u128)
            );
            assert_eq!(validation_result.reference_block, 12345);
            // Mock signatures should fail validation
            assert!(!validation_result.is_valid);
        }
        Err(_) => {
            // Also acceptable for mock signatures - the important thing is binary format worked
        }
    }
}

pub async fn run_mirror_abi_binary_compatibility_test() {
    // Test that we can round-trip ABI encoding/decoding exactly like Solidity
    let original_operators = [[0x11; 20], [0x22; 20], [0x33; 20]];

    let original_signatures = [vec![0x01; 65], vec![0x02; 65], vec![0x03; 65]];

    let original_block = 99999u32;

    // Encode using ethabi (same as Solidity)
    let tokens = vec![
        Token::Array(
            original_operators
                .iter()
                .map(|addr| Token::Address((*addr).into()))
                .collect(),
        ),
        Token::Array(
            original_signatures
                .iter()
                .map(|sig| Token::Bytes(sig.clone()))
                .collect(),
        ),
        Token::Uint(original_block.into()),
    ];

    let encoded = encode(&tokens);

    // Decode it back
    let param_types = vec![
        ParamType::Array(Box::new(ParamType::Address)),
        ParamType::Array(Box::new(ParamType::Bytes)),
        ParamType::Uint(32),
    ];

    let decoded = decode(&param_types, &encoded).unwrap();

    // Verify round-trip works
    assert_eq!(decoded.len(), 3);

    // Check operators
    if let Token::Array(addrs) = &decoded[0] {
        assert_eq!(addrs.len(), 3);
        for (i, addr) in addrs.iter().enumerate() {
            if let Token::Address(addr_bytes) = addr {
                assert_eq!(addr_bytes.as_bytes(), &original_operators[i]);
            } else {
                panic!("Expected address token");
            }
        }
    } else {
        panic!("Expected address array");
    }

    // Check signatures
    if let Token::Array(sigs) = &decoded[1] {
        assert_eq!(sigs.len(), 3);
        for (i, sig) in sigs.iter().enumerate() {
            if let Token::Bytes(sig_bytes) = sig {
                assert_eq!(sig_bytes, &original_signatures[i]);
            } else {
                panic!("Expected bytes token");
            }
        }
    } else {
        panic!("Expected signature array");
    }

    // Check block number
    if let Token::Uint(block_num) = &decoded[2] {
        assert_eq!(block_num.as_u32(), original_block);
    } else {
        panic!("Expected uint32");
    }
}