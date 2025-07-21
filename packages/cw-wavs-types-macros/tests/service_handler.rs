use cosmwasm_schema::{cw_serde, schema_for, QueryResponses};
use cw_wavs_types::service_handler::{ServiceHandlerExecuteMsg, ServiceHandlerQueryMsg};

use cw_wavs_types_macros::{service_handler_execute, service_handler_query};

mod helpers;
use helpers::extract_enum_variants_from_schema;

#[service_handler_query]
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
fn service_handler_query_derive() {
    let _test = TestQueryMsg::Foo {
        bar: "baz".to_string(),
    };

    let test = TestQueryMsg::ServiceManager {};

    match test {
        TestQueryMsg::Foo { .. } | TestQueryMsg::Bar(_) | TestQueryMsg::Baz => "yay",
        TestQueryMsg::ServiceManager {} => "yay",
    };
}

/// Test that the macro-generated enum includes all variants from the exported enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
#[test]
fn service_handler_query_macro_synced() {
    let required_variants = variants_of_enum!(ServiceHandlerQueryMsg);
    let macro_generated_variants = variants_of_enum!(TestQueryMsg);

    // Ensure all variants from the original enum are present in the macro-generated enum
    for variant in &required_variants {
        assert!(
            macro_generated_variants.contains(variant),
            "Macro-generated enum is missing variant '{variant}' from ServiceHandlerQueryMsg. \
                 Required variants: {required_variants:?}, Macro-generated variants: {macro_generated_variants:?}"
        );
    }

    println!("ServiceHandlerQueryMsg variants: {required_variants:?}");
    println!("Macro-generated variants: {macro_generated_variants:?}");
}

#[service_handler_execute]
#[allow(dead_code)]
#[cw_serde]
enum TestExecuteMsg {
    Foo { bar: String },
    Bar(u32),
    Baz,
}

#[test]
fn service_handler_execute_derive() {
    let test = TestExecuteMsg::Foo {
        bar: "baz".to_string(),
    };

    match test {
        TestExecuteMsg::Foo { .. } | TestExecuteMsg::Bar(_) | TestExecuteMsg::Baz => "yay",
        TestExecuteMsg::HandleSignedEnvelope { .. } => "yay",
    };
}

/// Test that the macro-generated enum includes all variants from the exported enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
#[test]
fn service_handler_execute_macro_synced() {
    let required_variants = variants_of_enum!(ServiceHandlerExecuteMsg);
    let macro_generated_variants = variants_of_enum!(TestExecuteMsg);

    // Ensure all variants from the original enum are present in the macro-generated enum
    for variant in &required_variants {
        assert!(
            macro_generated_variants.contains(variant),
            "Macro-generated enum is missing variant '{variant}' from ServiceHandlerExecuteMsg. \
                 Required variants: {required_variants:?}, Macro-generated variants: {macro_generated_variants:?}"
        );
    }

    println!("ServiceHandlerExecuteMsg variants: {required_variants:?}");
    println!("Macro-generated variants: {macro_generated_variants:?}");
}
