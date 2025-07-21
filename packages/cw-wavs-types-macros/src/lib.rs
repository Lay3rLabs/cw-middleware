mod helpers;

use helpers::merge_variants;

use proc_macro::TokenStream;
use quote::quote;

/// Add the required service handler query msg variants to the enum. This must be the first macro attribute in the enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
///
/// # Example
///
/// ```
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cw_wavs_types_macros::service_handler_query;
///
/// #[service_handler_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// pub enum QueryMsg {
///     #[returns(u32)]
///     Foo {
///         bar: String,
///     },
///     #[returns(u32)]
///     Bar {},
/// }
/// ```
#[proc_macro_attribute]
pub fn service_handler_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum ServiceHandlerQueryMsg {
                /// Returns the service manager address
                #[returns(::std::string::String)]
                ServiceManager {},
            }
        }
        .into(),
    )
}

/// Add the required service handler execute msg variants to the enum. This must be the first macro attribute in the enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
///
/// # Example
///
/// ```
/// use cosmwasm_schema::cw_serde;
/// use cw_wavs_types_macros::service_handler_execute;
///
/// #[service_handler_execute]
/// #[cw_serde]
/// pub enum ExecuteMsg {
///     DoSomething {
///         foo: u32,
///     },
///     DoSomethingElse {
///         bar: String,
///     },
/// }
/// ```
#[proc_macro_attribute]
pub fn service_handler_execute(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum ServiceHandlerExecuteMsg {
                /// Handles a signed envelope
                HandleSignedEnvelope {
                    /// The signed envelope
                    envelope: ::cw_wavs_types::Envelope,
                    /// The signature data
                    signature_data: ::cw_wavs_types::SignatureData,
                },
            }
        }
        .into(),
    )
}

/// Add the required service manager query msg variants to the enum. This must be the first macro attribute in the enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
///
/// # Example
///
/// ```
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cw_wavs_types_macros::service_manager_query;
///
/// #[service_manager_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// pub enum QueryMsg {
///     #[returns(u32)]
///     Foo {
///         bar: String,
///     },
///     #[returns(u32)]
///     Bar {},
/// }
/// ```
#[proc_macro_attribute]
pub fn service_manager_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum ServiceManagerQueryMsg {
                /// Gets the operator's current weight
                #[returns(::cosmwasm_std::Uint256)]
                OperatorWeight {
                    /// The address of the operator
                    operator: ::std::string::String,
                },
                /// Validates a signed envelope
                #[returns(())]
                Validate {
                    /// The envelope containing the data
                    envelope: ::cw_wavs_types::Envelope,
                    /// The signature data
                    signature_data: ::cw_wavs_types::SignatureData,
                },
                /// Returns the service URI
                #[returns(::std::string::String)]
                ServiceUri {},
                /// Returns the latest operator address associated with a signing key
                #[returns(::std::option::Option<::std::string::String>)]
                LatestOperatorForSigningKey {
                    /// The address of the signing key
                    address: ::std::string::String,
                },
                /// Returns the allocation manager address
                #[returns(::std::string::String)]
                AllocationManager {},
                /// Returns the delegation manager address
                #[returns(::std::string::String)]
                DelegationManager {},
                /// Returns the stake registry address
                #[returns(::std::string::String)]
                StakeRegistry {},
            }
        }
        .into(),
    )
}

/// Add the required service manager execute msg variants to the enum. This must be the first macro attribute in the enum. Macros cannot extract the variants from the original enum to add to the desired enum, so the macro requires redundant definitions.
///
/// # Example
///
/// ```
/// use cosmwasm_schema::cw_serde;
/// use cw_wavs_types_macros::service_manager_execute;
///
/// #[service_manager_execute]
/// #[cw_serde]
/// pub enum ExecuteMsg {
///     DoSomething {
///         foo: u32,
///     },
///     DoSomethingElse {
///         bar: String,
///     },
/// }
/// ```
#[proc_macro_attribute]
pub fn service_manager_execute(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum ServiceManagerExecuteMsg {
                /// Sets the service URI
                SetServiceUri {
                    /// The new service URI
                    service_uri: ::std::string::String,
                },
            }
        }
        .into(),
    )
}
