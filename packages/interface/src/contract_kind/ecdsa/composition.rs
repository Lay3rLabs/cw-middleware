use crate::{EcdsaServiceHandlerExecClient, EcdsaServiceHandlerQueryClient, EcdsaServiceManagerExecClient, EcdsaServiceManagerQueryClient};

// Convenience wrapper, automatically implemented
pub trait EcdsaServiceClient {
    type EcdsaServiceHandlerClient: EcdsaServiceHandlerExecClient + EcdsaServiceHandlerQueryClient;
    type EcdsaServiceManagerClient: EcdsaServiceManagerExecClient + EcdsaServiceManagerQueryClient;

    fn bls_handler(&self) -> &Self::EcdsaServiceHandlerClient;
    fn bls_manager(&self) -> &Self::EcdsaServiceManagerClient;
}

// blanket implementations
impl <T> EcdsaServiceClient for T
where
    T: EcdsaServiceHandlerExecClient + EcdsaServiceHandlerQueryClient + EcdsaServiceManagerExecClient + EcdsaServiceManagerQueryClient,
{
    type EcdsaServiceHandlerClient = T;
    type EcdsaServiceManagerClient = T;

    fn bls_handler(&self) -> &Self::EcdsaServiceHandlerClient {
        self
    }

    fn bls_manager(&self) -> &Self::EcdsaServiceManagerClient {
        self
    }
}