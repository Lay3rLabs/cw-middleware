use cosmwasm_schema::{cw_serde, schema_for, QueryResponses};
use cw_wavs_types::service_manager::{ServiceManagerExecuteMsg, ServiceManagerQueryMsg};

use cw_wavs_types_macros::{service_manager_execute, service_manager_query};

mod helpers;
use helpers::extract_enum_variants_from_schema;

#[service_manager_query]
#[allow(dead_code)]
#[cw_serde]
#[derive(QueryResponses)]
enum TestQueryMsg {
    #[returns(String)]
    Foo { bar: String },
    #[returns(String)]
    Bar(u32),
    #[returns(String)]
    Baz,
}

/// Successful compilation means the enum merge worked.
#[test]
fn service_manager_query_derive() {
    let _test = TestQueryMsg::Foo {
        bar: "baz".to_string(),
    };

    let test = TestQueryMsg::ServiceUri {};

    match test {
        TestQueryMsg::Foo { .. } | TestQueryMsg::Bar(_) | TestQueryMsg::Baz => "yay",
        TestQueryMsg::OperatorWeight { .. }
        | TestQueryMsg::Validate { .. }
        | TestQueryMsg::ServiceUri { .. }
        | TestQueryMsg::LatestOperatorForSigningKey { .. }
        | TestQueryMsg::AllocationManager { .. }
        | TestQueryMsg::DelegationManager { .. }
        | TestQueryMsg::StakeRegistry { .. } => "yay",
    };
}

#[service_manager_execute]
#[allow(dead_code)]
#[cw_serde]
enum TestExecuteMsg {
    Foo { bar: String },
    Bar(u32),
    Baz,
}

/// Successful compilation means the enum merge worked.
#[test]
fn service_manager_execute_derive() {
    let test = TestExecuteMsg::Foo {
        bar: "baz".to_string(),
    };

    match test {
        TestExecuteMsg::Foo { .. } | TestExecuteMsg::Bar(_) | TestExecuteMsg::Baz => "yay",
        TestExecuteMsg::SetServiceUri { .. } => "yay",
    };
}

/// Test that the macro-generated enum includes all variants from the exported enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
#[test]
fn service_manager_query_macro_synced() {
    let required_variants = variants_of_enum!(ServiceManagerQueryMsg);
    let macro_generated_variants = variants_of_enum!(TestQueryMsg);

    // Ensure all variants from the original enum are present in the macro-generated enum
    for variant in &required_variants {
        assert!(
            macro_generated_variants.contains(variant),
            "Macro-generated enum is missing variant '{variant}' from ServiceManagerQueryMsg. \
                 Required variants: {required_variants:?}, Macro-generated variants: {macro_generated_variants:?}"
        );
    }

    println!("ServiceManagerQueryMsg variants: {required_variants:?}");
    println!("Macro-generated variants: {macro_generated_variants:?}");
}

/// Test that the macro-generated enum includes all variants from the exported enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
#[test]
fn service_manager_execute_macro_synced() {
    let required_variants = variants_of_enum!(ServiceManagerExecuteMsg);
    let macro_generated_variants = variants_of_enum!(TestExecuteMsg);

    // Ensure all variants from the original enum are present in the macro-generated enum
    for variant in &required_variants {
        assert!(
            macro_generated_variants.contains(variant),
            "Macro-generated enum is missing variant '{variant}' from ServiceManagerExecuteMsg. \
                 Required variants: {required_variants:?}, Macro-generated variants: {macro_generated_variants:?}"
        );
    }

    println!("ServiceManagerExecuteMsg variants: {required_variants:?}");
    println!("Macro-generated variants: {macro_generated_variants:?}");
}
