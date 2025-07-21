use cosmwasm_std::{Addr, Event};
use cw2::ContractVersion;
use cw_multi_test::Executor;
use cw_wavs_types::{Envelope, SignatureData};

use crate::{
    msg::{ExecuteMsg, QueryMsg},
    test::suite::{Suite, OWNER},
};

#[test]
fn test_handle_signed_envelope_success() {
    let mut suite = Suite::new();

    let msg = ExecuteMsg::HandleSignedEnvelope {
        envelope: Envelope::default(),
        signature_data: SignatureData::default(),
    };

    let res = suite
        .app
        .execute_contract(
            Addr::unchecked(OWNER),
            suite.wavs_counter_success.clone(),
            &msg,
            &[],
        )
        .unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute")
                .add_attribute("_contract_address", suite.wavs_counter_success.to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", suite.wavs_counter_success.to_string())
                .add_attribute("action", "handle_signed_envelope")
                .add_attribute("counter", "1"),
        ]
    );
}

#[test]
fn test_handle_signed_envelope_error() {
    let mut suite = Suite::new();

    let msg = ExecuteMsg::HandleSignedEnvelope {
        envelope: Envelope::default(),
        signature_data: SignatureData::default(),
    };

    let error = suite
        .app
        .execute_contract(
            Addr::unchecked(OWNER),
            suite.wavs_counter_error.clone(),
            &msg,
            &[],
        )
        .unwrap_err();

    assert!(error.to_string().contains("validation error"));
}

#[test]
fn test_query_service_manager() {
    let suite = Suite::new();

    let res: String = suite
        .app
        .wrap()
        .query_wasm_smart(&suite.wavs_counter_success, &QueryMsg::ServiceManager {})
        .unwrap();

    assert_eq!(res, suite.service_manager_success.to_string());

    let res: String = suite
        .app
        .wrap()
        .query_wasm_smart(&suite.wavs_counter_error, &QueryMsg::ServiceManager {})
        .unwrap();

    assert_eq!(res, suite.service_manager_error.to_string());
}

#[test]
fn test_query_info() {
    let suite = Suite::new();

    let res: ContractVersion = suite
        .app
        .wrap()
        .query_wasm_smart(&suite.wavs_counter_success, &QueryMsg::Info {})
        .unwrap();

    assert_eq!(
        res,
        ContractVersion {
            contract: "crates.io:wavs-counter-handler".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    );
}
