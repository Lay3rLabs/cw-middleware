use crate::{BlsServiceHandlerExecClient, BlsServiceHandlerQueryClient, BlsServiceManagerExecClient, BlsServiceManagerQueryClient};

// Convenience wrapper, automatically implemented
pub trait BlsServiceClient {
    type BlsServiceHandlerClient: BlsServiceHandlerExecClient + BlsServiceHandlerQueryClient;
    type BlsServiceManagerClient: BlsServiceManagerExecClient + BlsServiceManagerQueryClient;

    fn bls_handler(&self) -> &Self::BlsServiceHandlerClient;
    fn bls_manager(&self) -> &Self::BlsServiceManagerClient;
}

// blanket implementations
impl <T> BlsServiceClient for T
where
    T: BlsServiceHandlerExecClient + BlsServiceHandlerQueryClient + BlsServiceManagerExecClient + BlsServiceManagerQueryClient,
{
    type BlsServiceHandlerClient = T;
    type BlsServiceManagerClient = T;

    fn bls_handler(&self) -> &Self::BlsServiceHandlerClient {
        self
    }

    fn bls_manager(&self) -> &Self::BlsServiceManagerClient {
        self
    }
}